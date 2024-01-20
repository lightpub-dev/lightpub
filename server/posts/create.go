package posts

import (
	"context"
	"errors"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/utils"
	"gorm.io/gorm"
)

type PostType int

const (
	PostTypeNormal PostType = iota
	PostTypeReply
	PostTypeRepost
	PostTypeQuote
)

var (
	ErrReplyOrRepostTargetNotFound = errors.New("reply or repost target not found")
	ErrRepostHasBody               = errors.New("repost has body")
	ErrAlreadyReposted             = errors.New("already reposted")
)

type CreateRequest struct {
	PosterID       db.UUID
	PosterUsername string
	Content        *string // should be nil when reposting
	Privacy        PrivacyType

	Poll *models.PostPollRequest

	ReplyToPostID *db.UUID // should be non-nil when replying
	RepostID      *db.UUID // should be non-nil when reposting or quoting
}

type CreateResponse struct {
	PostID string
}

func checkRepostable(ctx context.Context, conn db.DBConn, postID db.UUID) (bool, error) {
	// check if post is public or unlisted
	var privacy string
	err := conn.DB().Model(&db.Post{}).Where("id = ?", postID).Select("privacy").Find(&privacy).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, ErrReplyOrRepostTargetNotFound
		}
		return false, err
	}

	if privacy != string(PrivacyPublic) && privacy != string(PrivacyUnlisted) {
		return false, nil
	}
	return true, nil
}

func findOriginalPostID(ctx context.Context, conn db.DBConn, postID db.UUID) (db.UUID, error) {
	var originalPost db.Post
	err := conn.DB().Model(&db.Post{}).Select("repost_of", "content").Where("id=UUID_TO_BIN(?)", postID).First(&originalPost).Error
	if err != nil {
		return db.UUID{}, err
	}

	if !originalPost.RepostOfID.Valid {
		return postID, nil
	}

	if originalPost.Content != nil {
		return postID, nil
	}

	return findOriginalPostID(ctx, conn, originalPost.RepostOfID.UUID)
}

func FindOriginalPostID(ctx context.Context, conn db.DBConn, postID db.UUID) (db.UUID, error) {
	return findOriginalPostID(ctx, conn, postID)
}

// checkIfReposted checks if a post is reposted by a user
func checkIfReposted(ctx context.Context, conn db.DBConn, repostedID db.UUID, reposterID db.UUID) (bool, error) {
	// first, check if repostedID is a repost
	var repostOf db.Post
	err := conn.DB().Model(&db.Post{}).Select("repost_of_id", "content").Where("id = ?", repostedID).Find(&repostOf).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, ErrReplyOrRepostTargetNotFound
		}
		return false, err
	}

	if repostOf.RepostOfID.Valid && repostOf.Content == nil {
		// repostOf is a repost
		return checkIfReposted(ctx, conn, repostOf.RepostOfID.UUID, reposterID)
	}

	var count int64
	err = conn.DB().Model(&db.Post{}).Where("repost_of_id = ? AND poster_id = ? AND content IS NULL", repostedID, reposterID).Count(&count).Error
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func CreatePost(ctx context.Context, conn db.DBConn, post CreateRequest) (*CreateResponse, error) {
	postID, err := utils.GenerateUUID()
	if err != nil {
		return nil, err
	}

	currentTime := time.Now()
	var dbPost db.Post
	dbPost.ID = db.UUID(postID)
	dbPost.PosterID = post.PosterID
	dbPost.InsertedAt = currentTime
	dbPost.CreatedAt = currentTime
	dbPost.Privacy = string(post.Privacy)

	// var postType PostType

	if post.RepostID != nil {
		repostID, err := findOriginalPostID(ctx, conn, *post.RepostID)
		post.RepostID = &repostID
		if err != nil {
			return nil, err
		}
	}

	if post.ReplyToPostID != nil {
		ReplyToPostID, err := findOriginalPostID(ctx, conn, *post.ReplyToPostID)
		post.ReplyToPostID = &ReplyToPostID
		if err != nil {
			return nil, err
		}
	}

	if post.RepostID != nil && post.Content == nil {
		// Repost
		repostable, err := checkRepostable(ctx, conn, *post.RepostID)
		if err != nil {
			return nil, err
		}
		if !repostable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		// check if already reposted
		reposted, err := checkIfReposted(ctx, conn, *post.RepostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if reposted {
			return nil, ErrAlreadyReposted
		}

		dbPost.Content = nil
		dbPost.RepostOfID = post.RepostID.AsNullable()

		// postType = PostTypeRepost
	} else if post.RepostID != nil && post.Content != nil {
		// Quote
		quotable, err := checkRepostable(ctx, conn, *post.RepostID) // condition is same as repost
		if err != nil {
			return nil, err
		}
		if !quotable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content = post.Content
		dbPost.RepostOfID = post.RepostID.AsNullable()

		// postType = PostTypeQuote
	} else if post.ReplyToPostID != nil {
		// Reply
		// check if post is visible to poster
		visible, err := IsPostVisibleToUser(ctx, conn, *post.ReplyToPostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if !visible {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content = post.Content
		dbPost.ReplyToID = post.ReplyToPostID.AsNullable()

		// postType = PostTypeReply
	} else {
		// Normal post
		dbPost.Content = post.Content

		// postType = PostTypeNormal
	}

	// insert into db
	tx := conn.DB().Begin()
	defer tx.Rollback()

	// generate poll id if any
	// TODO: poll
	// var pollID string
	// if post.Poll != nil {
	// 	pollID, err = utils.GenerateUUIDString()
	// 	if err != nil {
	// 		return nil, err
	// 	}
	// 	dbPost.PollID = &pollID
	// }

	// insert Post
	if err := tx.Create(&dbPost).Error; err != nil {
		return nil, err
	}

	// insert Hashtags (if any)
	if post.Content != nil {
		hashtags := FindHashtags(*post.Content)
		for _, hashtag := range hashtags {
			if err := tx.Create(&db.PostHashtag{
				PostID:      dbPost.ID,
				HashtagName: hashtag,
			}).Error; err != nil {
				return nil, err
			}
		}
	}

	// insert Poll (if any)
	if post.Poll != nil {
		// TODO: poll
		// if postType == PostTypeRepost {
		// 	return nil, ErrRepostHasBody
		// }

		// dbPoll := models.PostPoll{
		// 	ID:            pollID,
		// 	AllowMultiple: post.Poll.AllowMultiple,
		// 	Due:           post.Poll.Due,
		// }
		// _, err = tx.NamedExecContext(ctx, "INSERT INTO PostPoll (id,allow_multiple,due) VALUES (UUID_TO_BIN(:id),:allow_multiple,:due)", dbPoll)
		// if err != nil {
		// 	return nil, err
		// }

		// // insert PollChoices
		// for _, choice := range post.Poll.Choices {
		// 	_, err = tx.ExecContext(ctx, "INSERT INTO PollChoice (poll_id,title,count) VALUES (UUID_TO_BIN(?),?,0)", pollID, choice)
		// 	if err != nil {
		// 		return nil, err
		// 	}
		// }
	}

	// insert mentions (if any)
	// TODO: mention
	// mentions := []string{}
	// if post.Content != nil {
	// 	mentions = FindMentions(*post.Content)
	// 	for _, mention := range mentions {

	// 		if err := tx.Create(&db.PostMention{
	// 			PostID: dbPost.ID,
	// 			TargetUserID: mention,
	// 		}); err != nil {
	// 			return nil, err
	// 		}
	// 	}
	// }

	// commit
	err = tx.Commit().Error
	if err != nil {
		return nil, err
	}

	// TODO: publish to timeline
	// TODO: this should be done asynchronously

	return &CreateResponse{PostID: postID.String()}, nil
}

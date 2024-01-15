package posts

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/utils"
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
	PosterID       string
	PosterUsername string
	Content        *string // should be when reposting
	Privacy        PrivacyType

	Poll *models.PostPollRequest

	ReplyToPostID string // should be non-empty when replying
	RepostID      string // should be non-empty when reposting or quoting
}

type CreateResponse struct {
	PostID string
}

func checkRepostable(ctx context.Context, conn db.DBConn, postId string) (bool, error) {
	// check if post is public or unlisted
	var privacy string
	err := conn.DB().GetContext(ctx, &privacy, "SELECT privacy FROM Post WHERE id=UUID_TO_BIN(?)", postId)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, ErrReplyOrRepostTargetNotFound
		}
		return false, err
	}

	if privacy != string(PrivacyPublic) && privacy != string(PrivacyUnlisted) {
		return false, nil
	}
	return true, nil
}

func findOriginalPostID(ctx context.Context, conn db.DBConn, postID string) (string, error) {
	var originalPostID models.Post
	err := conn.DB().GetContext(ctx, &originalPostID, "SELECT BIN_TO_UUID(repost_of) AS repost_of, content FROM Post WHERE id=UUID_TO_BIN(?)", postID)
	if err != nil {
		return "", err
	}

	if originalPostID.RepostOf == nil {
		return postID, nil
	}

	if originalPostID.Content != nil {
		return postID, nil
	}

	return findOriginalPostID(ctx, conn, *originalPostID.RepostOf)
}

// checkIfReposted checks if a post is reposted by a user
func checkIfReposted(ctx context.Context, conn db.DBConn, repostedID string, reposterID string) (bool, error) {
	// first, check if repostedID is a repost
	var repostOf models.Post
	err := conn.DB().GetContext(ctx, &repostOf, "SELECT BIN_TO_UUID(repost_of) AS repost_of, content FROM Post WHERE id=UUID_TO_BIN(?)", repostedID)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, ErrReplyOrRepostTargetNotFound
		}
		return false, err
	}

	if repostOf.RepostOf != nil && repostOf.Content == nil {
		// repostOf is a repost
		return checkIfReposted(ctx, conn, *repostOf.RepostOf, reposterID)
	}

	var count int
	err = conn.DB().GetContext(ctx, &count, "SELECT COUNT(*) FROM Post WHERE repost_of=UUID_TO_BIN(?) AND poster_id=UUID_TO_BIN(?) AND content IS NULL", repostedID, reposterID)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func CreatePost(ctx context.Context, conn db.DBConn, post CreateRequest) (*CreateResponse, error) {
	postID, err := utils.GenerateUUIDString()
	if err != nil {
		return nil, err
	}

	currentTime := time.Now()
	var dbPost models.Post
	dbPost.ID = postID
	dbPost.PosterID = post.PosterID
	dbPost.InsertedAt = currentTime
	dbPost.CreatedAt = currentTime
	dbPost.Privacy = string(post.Privacy)

	var postType PostType

	if post.RepostID != "" {
		post.RepostID, err = findOriginalPostID(ctx, conn, post.RepostID)
		if err != nil {
			return nil, err
		}
	}

	if post.ReplyToPostID != "" {
		post.ReplyToPostID, err = findOriginalPostID(ctx, conn, post.ReplyToPostID)
		if err != nil {
			return nil, err
		}
	}

	if post.RepostID != "" && post.Content == nil {
		// Repost
		repostable, err := checkRepostable(ctx, conn, post.RepostID)
		if err != nil {
			return nil, err
		}
		if !repostable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		// check if already reposted
		reposted, err := checkIfReposted(ctx, conn, post.RepostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if reposted {
			return nil, ErrAlreadyReposted
		}

		dbPost.Content = nil
		dbPost.RepostOf = &post.RepostID

		postType = PostTypeRepost
	} else if post.RepostID != "" && post.Content != nil {
		// Quote
		quotable, err := checkRepostable(ctx, conn, post.RepostID) // condition is same as repost
		if err != nil {
			return nil, err
		}
		if !quotable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content = post.Content
		dbPost.RepostOf = &post.RepostID

		postType = PostTypeQuote
	} else if post.ReplyToPostID != "" {
		// Reply
		// check if post is visible to poster
		visible, err := IsPostVisibleToUser(ctx, conn, post.ReplyToPostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if !visible {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content = post.Content
		dbPost.ReplyTo = &post.ReplyToPostID

		postType = PostTypeReply
	} else {
		// Normal post
		dbPost.Content = post.Content

		postType = PostTypeNormal
	}

	// insert into db
	tx, err := conn.DB().Beginx()
	if err != nil {
		return nil, err
	}
	defer tx.Rollback()

	// generate poll id if any
	var pollID string
	if post.Poll != nil {
		pollID, err = utils.GenerateUUIDString()
		if err != nil {
			return nil, err
		}
		dbPost.PollID = &pollID
	}

	// insert Post
	_, err = tx.NamedExecContext(ctx, "INSERT INTO Post (id,poster_id,content,inserted_at,created_at,privacy,reply_to,repost_of,poll_id) VALUES (UUID_TO_BIN(:id),UUID_TO_BIN(:poster_id),:content,:inserted_at,:created_at,:privacy,UUID_TO_BIN(:reply_to),UUID_TO_BIN(:repost_of),UUID_TO_BIN(:poll_id))", dbPost)
	if err != nil {
		return nil, err
	}

	// insert Hashtags (if any)
	if post.Content != nil {
		hashtags := FindHashtags(*post.Content)
		for _, hashtag := range hashtags {
			_, err = tx.ExecContext(ctx, "INSERT INTO PostHashtag (post_id,hashtag_name) VALUES (UUID_TO_BIN(?),?)", postID, hashtag)
			if err != nil {
				return nil, err
			}
		}
	}

	// insert Poll (if any)
	if post.Poll != nil {
		if postType == PostTypeRepost {
			return nil, ErrRepostHasBody
		}

		dbPoll := models.PostPoll{
			ID:            pollID,
			AllowMultiple: post.Poll.AllowMultiple,
			Due:           post.Poll.Due,
		}
		_, err = tx.NamedExecContext(ctx, "INSERT INTO PostPoll (id,allow_multiple,due) VALUES (UUID_TO_BIN(:id),:allow_multiple,:due)", dbPoll)
		if err != nil {
			return nil, err
		}

		// insert PollChoices
		for _, choice := range post.Poll.Choices {
			_, err = tx.ExecContext(ctx, "INSERT INTO PollChoice (poll_id,title,count) VALUES (UUID_TO_BIN(?),?,0)", pollID, choice)
			if err != nil {
				return nil, err
			}
		}
	}

	// insert mentions (if any)
	mentions := []string{}
	if post.Content != nil {
		mentions = FindMentions(*post.Content)
		for _, mention := range mentions {
			_, err = tx.ExecContext(ctx, "INSERT INTO PostMention (post_id,target_user_id) VALUES (UUID_TO_BIN(?),UUID_TO_BIN(?))", postID, mention)
			if err != nil {
				return nil, err
			}
		}
	}

	// commit
	err = tx.Commit()
	if err != nil {
		return nil, err
	}

	// TODO: publish to timeline
	// TODO: this should be done asynchronously

	return &CreateResponse{PostID: postID}, nil
}

package posts

import (
	"errors"
	"time"

	"github.com/lightpub-dev/lightpub/db"
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

type PostCreateService interface {
	CreatePost(post CreateRequest) (*CreateResponse, error)
	// FindOriginalPostID(postID db.UUID) (db.UUID, error)
}

type DBPostCreateService struct {
	conn           db.DBConn
	postVisibility PostVisibilityService
	fetch          PostFetchService
}

func ProvideDBPostCreateService(conn db.DBConn, postVisibility PostVisibilityService, fetch PostFetchService) *DBPostCreateService {
	return &DBPostCreateService{conn, postVisibility, fetch}
}

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

	// Poll *models.PostPollRequest

	ReplyToPostID *db.UUID // should be non-nil when replying
	RepostID      *db.UUID // should be non-nil when reposting or quoting
}

type CreateResponse struct {
	PostID string
}

func (s *DBPostCreateService) checkRepostable(postID db.UUID) (bool, error) {
	conn := s.conn.DB

	// check if post is public or unlisted
	var privacy string
	err := conn.Model(&db.Post{}).Where("id = ?", postID).Select("privacy").Find(&privacy).Error
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

func (s *DBPostFetchService) findOriginalPostID(postID db.UUID) (db.UUID, error) {
	conn := s.conn.DB

	var originalPost db.Post
	err := conn.Model(&db.Post{}).Select("repost_of", "content").Where("id=UUID_TO_BIN(?)", postID).First(&originalPost).Error
	if err != nil {
		return db.UUID{}, err
	}

	if !originalPost.RepostOfID.Valid {
		return postID, nil
	}

	if originalPost.Content.Valid {
		return postID, nil
	}

	return s.findOriginalPostID(originalPost.RepostOfID.UUID)
}

func (s *DBPostFetchService) FindOriginalPostID(postID db.UUID) (db.UUID, error) {
	return s.findOriginalPostID(postID)
}

// checkIfReposted checks if a post is reposted by a user
func (s *DBPostCreateService) checkIfReposted(repostedID db.UUID, reposterID db.UUID) (bool, error) {
	conn := s.conn.DB

	// first, check if repostedID is a repost
	var repostOf db.Post
	err := conn.Model(&db.Post{}).Select("repost_of_id", "content").Where("id = ?", repostedID).Find(&repostOf).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, ErrReplyOrRepostTargetNotFound
		}
		return false, err
	}

	if repostOf.RepostOfID.Valid && !repostOf.Content.Valid {
		// repostOf is a repost
		return s.checkIfReposted(repostOf.RepostOfID.UUID, reposterID)
	}

	var count int64
	err = conn.Model(&db.Post{}).Where("repost_of_id = ? AND poster_id = ? AND content IS NULL", repostedID, reposterID).Count(&count).Error
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func (s *DBPostCreateService) CreatePost(post CreateRequest) (*CreateResponse, error) {
	conn := s.conn.DB

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
		repostID, err := s.fetch.FindOriginalPostID(*post.RepostID)
		post.RepostID = &repostID
		if err != nil {
			return nil, err
		}
	}

	if post.ReplyToPostID != nil {
		ReplyToPostID, err := s.fetch.FindOriginalPostID(*post.ReplyToPostID)
		post.ReplyToPostID = &ReplyToPostID
		if err != nil {
			return nil, err
		}
	}

	if post.RepostID != nil && post.Content == nil {
		// Repost
		repostable, err := s.checkRepostable(*post.RepostID)
		if err != nil {
			return nil, err
		}
		if !repostable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		// check if already reposted
		reposted, err := s.checkIfReposted(*post.RepostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if reposted {
			return nil, ErrAlreadyReposted
		}

		dbPost.Content.Valid = false
		dbPost.RepostOfID = post.RepostID.AsNullable()

		// postType = PostTypeRepost
	} else if post.RepostID != nil && post.Content != nil {
		// Quote
		quotable, err := s.checkRepostable(*post.RepostID) // condition is same as repost
		if err != nil {
			return nil, err
		}
		if !quotable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content.Valid = true
		dbPost.Content.String = *post.Content
		dbPost.RepostOfID = post.RepostID.AsNullable()

		// postType = PostTypeQuote
	} else if post.ReplyToPostID != nil {
		// Reply
		// check if post is visible to poster
		visible, err := s.postVisibility.IsPostVisibleToUser(*post.ReplyToPostID, post.PosterID)
		if err != nil {
			return nil, err
		}
		if !visible {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content.Valid = true
		dbPost.Content.String = *post.Content
		dbPost.ReplyToID = post.ReplyToPostID.AsNullable()

		// postType = PostTypeReply
	} else {
		// Normal post
		dbPost.Content.Valid = true
		dbPost.Content.String = *post.Content

		// postType = PostTypeNormal
	}

	// insert into db
	tx := conn.Begin()
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

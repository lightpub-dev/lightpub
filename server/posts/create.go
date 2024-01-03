package posts

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/jmoiron/sqlx"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/utils"
	"github.com/redis/go-redis/v9"
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

func checkRepostable(db *sqlx.DB, postId string) (bool, error) {
	// check if post is public or unlisted
	var privacy string
	err := db.Get(&privacy, "SELECT privacy FROM Post WHERE id=UUID_TO_BIN(?)", postId)
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

func CreatePost(ctx context.Context, db *sqlx.DB, rdb *redis.Client, post CreateRequest) (*CreateResponse, error) {
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

	if post.RepostID != "" && post.Content == nil {
		// Repost
		repostable, err := checkRepostable(db, post.RepostID)
		if err != nil {
			return nil, err
		}
		if !repostable {
			// hide existence of post
			return nil, ErrReplyOrRepostTargetNotFound
		}

		dbPost.Content = nil
		dbPost.RepostOf = &post.RepostID

		postType = PostTypeRepost
	} else if post.RepostID != "" && post.Content != nil {
		// Quote
		quotable, err := checkRepostable(db, post.RepostID) // condition is same as repost
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
		visible, err := IsPostVisibleToUser(ctx, db, post.ReplyToPostID, post.PosterID)
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
	tx, err := db.Beginx()
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
	_, err = tx.NamedExec("INSERT INTO Post (id,poster_id,content,inserted_at,created_at,privacy,reply_to,repost_of,poll_id) VALUES (UUID_TO_BIN(:id),UUID_TO_BIN(:poster_id),:content,:inserted_at,:created_at,:privacy,UUID_TO_BIN(:reply_to),UUID_TO_BIN(:repost_of),UUID_TO_BIN(:poll_id))", dbPost)
	if err != nil {
		return nil, err
	}

	// insert Hashtags (if any)
	hashtags := []string{}
	if post.Content != nil {
		hashtags = FindHashtags(*post.Content)
		for _, hashtag := range hashtags {
			_, err = tx.Exec("INSERT INTO PostHashtag (post_id,hashtag_name) VALUES (UUID_TO_BIN(?),?)", postID, hashtag)
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
		_, err = tx.NamedExec("INSERT INTO PostPoll (id,allow_multiple,due) VALUES (UUID_TO_BIN(:id),:allow_multiple,:due)", dbPoll)
		if err != nil {
			return nil, err
		}

		// insert PollChoices
		for _, choice := range post.Poll.Choices {
			_, err = tx.Exec("INSERT INTO PollChoice (poll_id,title,count) VALUES (UUID_TO_BIN(?),?,0)", pollID, choice)
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
			_, err = tx.Exec("INSERT INTO PostMention (post_id,target_user_id) VALUES (UUID_TO_BIN(?),UUID_TO_BIN(?))", postID, mention)
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

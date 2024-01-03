package posts

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

const (
	MaxPostExpandDepth = 1
)

type postWithUser struct {
	ID             string    `db:"id"`
	PosterID       string    `db:"poster_id"`
	PosterUsername string    `db:"poster_username"`
	PosterHost     string    `db:"poster_host"`
	Content        *string   `db:"content"`
	CreatedAt      time.Time `db:"created_at"`
	Privacy        string    `db:"privacy"`

	ReplyTo  *string `db:"reply_to"`
	RepostOf *string `db:"repost_of"`
	PollID   *string `db:"poll_id"`
}

func createPostURL(postID string) string {
	return fmt.Sprintf("%s/post/%s", config.BaseURL, postID)
}

func fillUserPostEntry(result *models.UserPostEntry, post postWithUser,
	replyTo interface{}, // *models.UserPostEntry || string || nil
	repostOf interface{}, // *models.UserPostEntry || string || nil
) {
	result.ID = post.ID
	result.Author.ID = post.PosterID
	result.Author.Username = post.PosterUsername
	result.Author.Host = post.PosterHost
	result.Content = post.Content
	result.CreatedAt = post.CreatedAt
	result.Privacy = post.Privacy

	result.ReplyTo = replyTo
	result.RepostOf = repostOf
}

// fetchSinglePostOrURL returns *models.UserPostEntry || string
func fetchSinglePostOrURL(ctx context.Context, tx db.DBOrTx, postID string, viewerUserID string, currentDepth int) (interface{}, error) {
	if currentDepth >= MaxPostExpandDepth {
		return createPostURL(postID), nil
	}

	post, err := fetchSinglePost(ctx, tx, postID, viewerUserID, currentDepth+1)
	if err != nil {
		return nil, err
	}

	if post == nil {
		return createPostURL(postID), nil
	}

	return post, nil
}

func FetchSinglePost(ctx context.Context, tx db.DBOrTx, postID string, viewerUserID string) (*models.UserPostEntry, error) {
	return fetchSinglePost(ctx, tx, postID, viewerUserID, 0)
}

func fetchSinglePost(ctx context.Context, tx db.DBOrTx, postID string, viewerUserID string, currentDepth int) (*models.UserPostEntry, error) {
	var post postWithUser
	err := tx.GetContext(ctx, &post, `
	SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
	FROM Post p
	INNER JOIN User u ON p.poster_id=u.id
	WHERE
		p.id=UUID_TO_BIN(?)
		AND p.scheduled_at IS NULL
	`, postID)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	// check if viewer can see this post
	if viewerUserID == "" {
		// visible if privacy is public or unlisted
		switch post.Privacy {
		case string(PrivacyPublic):
			fallthrough
		case string(PrivacyUnlisted):
			break
		default:
			return nil, nil
		}
	} else {
		visible, err := IsPostVisibleToUser(ctx, tx, postID, viewerUserID)
		if err != nil {
			return nil, err
		}
		if !visible {
			return nil, nil
		}
	}

	// TODO: fetch poll

	// fetch replied post
	var replyToPost interface{}
	if post.ReplyTo != nil {
		var err error
		replyToPost, err = fetchSinglePostOrURL(ctx, tx, *post.ReplyTo, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	// fetch reposted post
	var repostOfPost interface{}
	if post.RepostOf != nil {
		var err error
		repostOfPost, err = fetchSinglePostOrURL(ctx, tx, *post.RepostOf, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	result := &models.UserPostEntry{}
	fillUserPostEntry(result, post, replyToPost, repostOfPost)

	return result, nil
}

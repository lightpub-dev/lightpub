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
	PosterNickname string    `db:"poster_nickname"`
	Content        *string   `db:"content"`
	CreatedAt      time.Time `db:"created_at"`
	Privacy        string    `db:"privacy"`

	ReplyTo  *string `db:"reply_to"`
	RepostOf *string `db:"repost_of"`
	PollID   *string `db:"poll_id"`

	RepostedByMe *bool `db:"reposted_by_me"`
}

func CreatePostURL(postID string) string {
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
	result.Author.Nickname = post.PosterNickname
	result.Content = post.Content
	result.CreatedAt = post.CreatedAt
	result.Privacy = post.Privacy

	result.ReplyTo = replyTo
	result.RepostOf = repostOf

	result.RepostedByMe = post.RepostedByMe
}

// fetchSinglePostOrURL returns *models.UserPostEntry || string
func fetchSinglePostOrURL(ctx context.Context, conn db.DBConn, postID string, viewerUserID string, currentDepth int) (interface{}, error) {
	if currentDepth >= MaxPostExpandDepth {
		return CreatePostURL(postID), nil
	}

	post, err := FetchSinglePostWithDepth(ctx, conn, postID, viewerUserID, currentDepth+1)
	if err != nil {
		return nil, err
	}

	if post == nil {
		return CreatePostURL(postID), nil
	}

	return post, nil
}

func FetchSinglePost(ctx context.Context, conn db.DBConn, postID string, viewerUserID string) (*models.UserPostEntry, error) {
	return FetchSinglePostWithDepth(ctx, conn, postID, viewerUserID, 0)
}

func FetchSinglePostWithDepth(ctx context.Context, conn db.DBConn, postID string, viewerUserID string, currentDepth int) (*models.UserPostEntry, error) {
	var post postWithUser
	err := conn.DB().GetContext(ctx, &post, `
	SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,u.nickname AS poster_nickname,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(?) AND p2.content IS NULL)) AS reposted_by_me
	FROM Post p
	INNER JOIN User u ON p.poster_id=u.id
	WHERE
		p.id=UUID_TO_BIN(?)
		AND p.scheduled_at IS NULL
	`, viewerUserID, viewerUserID, postID)
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
		visible, err := IsPostVisibleToUser(ctx, conn, postID, viewerUserID)
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
		replyToPost, err = fetchSinglePostOrURL(ctx, conn, *post.ReplyTo, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	// fetch reposted post
	var repostOfPost interface{}
	if post.RepostOf != nil {
		var err error
		repostOfPost, err = fetchSinglePostOrURL(ctx, conn, *post.RepostOf, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	result := &models.UserPostEntry{}
	fillUserPostEntry(result, post, replyToPost, repostOfPost)

	FillCounts(ctx, conn, result)

	return result, nil
}

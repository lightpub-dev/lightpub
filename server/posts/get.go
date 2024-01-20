package posts

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/google/uuid"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

const (
	MaxPostExpandDepth = 1
)

type postWithUser struct {
	db.Post

	RepostedByMe   *bool
	FavoritedByMe  *bool
	BookmarkedByMe *bool
}

func CreatePostURL(postID db.UUID) string {
	return fmt.Sprintf("%s/post/%s", config.BaseURL, uuid.UUID(postID).String())
}

func fillUserPostEntry(result *models.UserPostEntry, post postWithUser,
	replyTo interface{}, // *models.UserPostEntry || string || nil
	repostOf interface{}, // *models.UserPostEntry || string || nil
) {
	result.ID = post.ID.String()
	result.Author.ID = post.Poster.ID.String()
	result.Author.Username = post.Poster.Username
	result.Author.Host = post.Poster.Host
	result.Author.Nickname = post.Poster.Nickname
	result.Content = post.Content
	result.CreatedAt = post.CreatedAt
	result.Privacy = post.Privacy

	result.ReplyTo = replyTo
	result.RepostOf = repostOf

	result.RepostedByMe = post.RepostedByMe
	result.FavoritedByMe = post.FavoritedByMe
	result.BookmarkedByMe = post.BookmarkedByMe
}

// fetchSinglePostOrURL returns *models.UserPostEntry || string
func fetchSinglePostOrURL(ctx context.Context, conn db.DBConn, postID db.UUID, viewerUserID db.UUID, currentDepth int) (interface{}, error) {
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

func FetchSinglePost(ctx context.Context, conn db.DBConn, postID db.UUID, viewerUserID db.UUID) (*models.UserPostEntry, error) {
	return FetchSinglePostWithDepth(ctx, conn, postID, viewerUserID, 0)
}

func FetchSinglePostWithDepth(ctx context.Context, conn db.DBConn, postID db.UUID, viewerUserID db.UUID, currentDepth int) (*models.UserPostEntry, error) {
	var post db.Post
	err := conn.DB().Find(&post, "id = ?", postID).Joins("JOIN users ON users.id = posts.poster_id").Error
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	var (
		repostedByMe, favoritedByMe, bookmarkedByMe *bool
	)

	if viewerUserID != (db.UUID{}) {
		var repostedByMeCount, favoritedByMeCount, bookmarkedByMeCount int64
		err := conn.DB().Model(&db.Post{}).Where("repost_of_id = ? AND poster_id = ? AND content IS NULL", postID, viewerUserID).Count(&repostedByMeCount).Error
		if err != nil {
			return nil, err
		}
		repostedByMe = new(bool)
		*repostedByMe = repostedByMeCount > 0

		err = conn.DB().Model(&db.PostFavorite{}).Where("post_id = ? AND user_id = ? AND is_bookmark = 0", postID, viewerUserID).Count(&favoritedByMeCount).Error
		if err != nil {
			return nil, err
		}
		favoritedByMe = new(bool)
		*favoritedByMe = favoritedByMeCount > 0

		err = conn.DB().Model(&db.PostFavorite{}).Where("post_id = ? AND user_id = ? AND is_bookmark = 1", postID, viewerUserID).Count(&bookmarkedByMeCount).Error
		if err != nil {
			return nil, err
		}
		bookmarkedByMe = new(bool)
		*bookmarkedByMe = bookmarkedByMeCount > 0
	}

	// check if viewer can see this post
	if viewerUserID == (db.UUID{}) {
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
		replyToPost, err = fetchSinglePostOrURL(ctx, conn, *post.ReplyToID, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	// fetch reposted post
	var repostOfPost interface{}
	if post.RepostOf != nil {
		var err error
		repostOfPost, err = fetchSinglePostOrURL(ctx, conn, *post.RepostOfID, viewerUserID, currentDepth)
		if err != nil {
			return nil, err
		}
	}

	result := &models.UserPostEntry{}
	fillUserPostEntry(result, postWithUser{
		Post:           post,
		RepostedByMe:   repostedByMe,
		FavoritedByMe:  favoritedByMe,
		BookmarkedByMe: bookmarkedByMe,
	}, replyToPost, repostOfPost)

	FillCounts(ctx, conn, result)

	return result, nil
}

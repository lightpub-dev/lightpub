package posts

import (
	"context"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/users"
)

type PrivacyType string

const (
	PrivacyPublic   PrivacyType = "public"
	PrivacyUnlisted PrivacyType = "unlisted"
	PrivacyFollower PrivacyType = "follower"
	PrivacyPrivate  PrivacyType = "private"
)

func IsPostVisibleToUser(ctx context.Context, conn db.DBConn, postId db.UUID, userId db.UUID) (bool, error) {
	var post db.Post
	err := conn.DB().Model(&db.Post{}).Select("poster_id", "privacy").Where("id = ?", postId).First(&post).Error
	if err != nil {
		return false, err
	}

	// if poster is the same as viewer, visible
	if post.PosterID == userId {
		return true, nil
	}

	switch PrivacyType(post.Privacy) {
	case PrivacyPublic:
		fallthrough
	case PrivacyUnlisted:
		return true, nil
	case PrivacyFollower:
		// check if user is followed by poster
		posterID := post.PosterID
		isFollowedBy, err := users.IsFollowedBy(ctx, conn, posterID, userId)
		if err != nil {
			return false, err
		}
		return isFollowedBy, nil
	case PrivacyPrivate:
		// check if user is in post's mention list
		var count int64
		err := conn.DB().Model(&db.PostMention{}).Where("post_id = ? AND target_user_id = ?", postId, userId).Count(&count).Error
		if err != nil {
			return false, err
		}
		return count > 0, nil
	default:
		return false, errors.New("invalid privacy")
	}
}

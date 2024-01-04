package posts

import (
	"context"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/users"
)

type PrivacyType string

const (
	PrivacyPublic   PrivacyType = "public"
	PrivacyUnlisted PrivacyType = "unlisted"
	PrivacyFollower PrivacyType = "follower"
	PrivacyPrivate  PrivacyType = "private"
)

func IsPostVisibleToUser(ctx context.Context, conn db.DBConn, postId string, userId string) (bool, error) {
	var post models.Post
	err := conn.DB().GetContext(ctx, &post, "SELECT BIN_TO_UUID(poster_id) AS poster_id,privacy FROM Post WHERE id=UUID_TO_BIN(?)", postId)
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
		var count int
		err := conn.DB().GetContext(ctx, &count, "SELECT COUNT(*) FROM PostMention WHERE post_id=UUID_TO_BIN(?) AND target_user_id=UUID_TO_BIN(?)", postId, userId)
		if err != nil {
			return false, err
		}
		return count > 0, nil
	default:
		return false, errors.New("invalid privacy")
	}
}

package posts

import (
	"errors"

	"github.com/jmoiron/sqlx"
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

func IsPostVisibleToUser(db *sqlx.DB, postId string, userId string) (bool, error) {
	var post models.Post
	err := db.Get(&post, "SELECT poster_id,privacy FROM Post WHERE id=UUID_TO_BIN(?)", postId)
	if err != nil {
		return false, err
	}

	switch PrivacyType(post.Privacy) {
	case PrivacyPublic:
		fallthrough
	case PrivacyUnlisted:
		return true, nil
	case PrivacyFollower:
		// check if user is followed by poster
		posterID := post.PosterID
		isFollowedBy, err := users.IsFollowedBy(db, posterID, userId)
		if err != nil {
			return false, err
		}
		return isFollowedBy, nil
	case PrivacyPrivate:
		// check if user is in post's mention list
		var count int
		err := db.Get(&count, "SELECT COUNT(*) FROM PostMention WHERE post_id=UUID_TO_BIN(?) AND target_user_id=UUID_TO_BIN(?)", postId, userId)
		if err != nil {
			return false, err
		}
		return count > 0, nil
	default:
		return false, errors.New("invalid privacy")
	}
}

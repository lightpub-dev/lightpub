package posts

import (
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/users"
)

type PostVisibilityService interface {
	IsPostVisibleToUser(postId db.UUID, userId db.UUID) (bool, error)
}

type DBPostVisibilityService struct {
	conn          db.DBConn
	followService users.UserFollowService
}

func ProvideDBPostVisibilityService(conn db.DBConn, followService users.UserFollowService) *DBPostVisibilityService {
	return &DBPostVisibilityService{conn, followService}
}

type PrivacyType string

const (
	PrivacyPublic   PrivacyType = "public"
	PrivacyUnlisted PrivacyType = "unlisted"
	PrivacyFollower PrivacyType = "follower"
	PrivacyPrivate  PrivacyType = "private"
)

func (s *DBPostVisibilityService) IsPostVisibleToUser(postId db.UUID, userId db.UUID) (bool, error) {
	conn := s.conn.DB

	var post db.Post
	err := conn.Model(&db.Post{}).Select("poster_id", "privacy").Where("id = ?", postId).First(&post).Error
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
		isFollowedBy, err := s.followService.IsFollowedBy(posterID, userId)
		if err != nil {
			return false, err
		}
		return isFollowedBy, nil
	case PrivacyPrivate:
		// check if user is in post's mention list
		var count int64
		err := conn.Model(&db.PostMention{}).Where("post_id = ? AND target_user_id = ?", postId, userId).Count(&count).Error
		if err != nil {
			return false, err
		}
		return count > 0, nil
	default:
		return false, errors.New("invalid privacy")
	}
}

package users

import (
	"database/sql"
	"errors"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
	"gorm.io/gorm"
)

type UserFollowService interface {
	IsFollowedBy(followerID db.UUID, followeeID db.UUID) (bool, error)
	FindFollowers(followeeID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error)
	FindFollowing(followerID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error)
	FindFollowersInboxes(followeeID db.UUID) ([]FollowerInbox, error)
	Follow(followerID db.UUID, followeeID db.UUID) error
	Unfollow(followerID db.UUID, followeeID db.UUID) error
}

type DBUserFollowService struct {
	conn       db.DBConn
	pubFollow  *PubFollowService
	userFinder UserFinderService
	idGetter   pub.IDGetterService
}

func ProvideDBUserFollowService(conn db.DBConn, pubFollow *PubFollowService, userFinder UserFinderService, idGetter pub.IDGetterService) *DBUserFollowService {
	return &DBUserFollowService{conn: conn, pubFollow: pubFollow, userFinder: userFinder, idGetter: idGetter}
}

type FollowerInbox struct {
	UserID      db.UUID `gorm:"column:id"`
	Inbox       sql.NullString
	SharedInbox sql.NullString
}

func (s *DBUserFollowService) IsFollowedBy(followerID db.UUID, followeeID db.UUID) (bool, error) {
	conn := s.conn.DB

	var count int64
	err := conn.Model(&db.UserFollow{}).Where("follower_id = ? AND followee_id = ?", followerID, followeeID).Count(&count).Error
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

type FollowerInfo struct {
	ID          string  `json:"id"`
	Username    string  `json:"username"`
	Host        string  `json:"host"`
	URL         *string `json:"url"` // always non-nil after fillInLocalURL
	Nickname    string  `json:"nickname"`
	Bio         string  `json:"bio"`
	IsFollowing bool    `json:"is_following"`
}

func CreateLocalUserURL(username string) string {
	return fmt.Sprintf("%s/user/%s", config.BaseURL, username)
}

func fillInLocalURL(follower *FollowerInfo) {
	if follower.URL == nil {
		localUrl := CreateLocalUserURL(follower.Username)
		follower.URL = &localUrl
	}
}

func (s *DBUserFollowService) FindFollowers(followeeID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	conn := s.conn.DB

	var (
		followers []FollowerInfo
		tx        *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("followee_id = ?", followeeID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("followee_id = ?", followeeID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC")
	if limit >= 0 {
		tx = tx.Limit(limit)
	}

	err := tx.Find(&followers).Error
	if err != nil {
		return nil, err
	}

	for i, follower := range followers {
		fillInLocalURL(&follower)
		followers[i] = follower
	}

	return followers, nil
}

func (s *DBUserFollowService) FindFollowing(followerID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	conn := s.conn.DB

	var (
		followings []FollowerInfo
		tx         *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("follower_id = ?", followerID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("follower_id = ?", followerID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC")
	if limit >= 0 {
		tx = tx.Limit(limit)
	}

	err := tx.Find(&followings).Error
	if err != nil {
		return nil, err
	}

	for i, follower := range followings {
		fillInLocalURL(&follower)
		followings[i] = follower
	}

	return followings, nil
}

func (s *DBUserFollowService) FindFollowersInboxes(followeeID db.UUID) ([]FollowerInbox, error) {
	conn := s.conn.DB

	var inboxes []FollowerInbox
	// TODO: follower が多すぎると IN の制限でエラーにならない?
	err := conn.Model(&db.User{}).Select("id, inbox, shared_inbox").Where("id IN (SELECT follower_id FROM user_follows WHERE followee_id = ?)", followeeID).Find(&inboxes).Error
	if err != nil {
		return nil, err
	}

	return inboxes, nil
}

func (s *DBUserFollowService) Follow(followerID db.UUID, followeeID db.UUID) error {
	conn := s.conn.DB

	followee, err := s.userFinder.FetchUserByID(followeeID)
	if err != nil {
		return err
	}
	follower, err := s.userFinder.FetchUserByID(followerID)
	if err != nil {
		return err
	}

	if !followee.Host.Valid {
		// local user
		follow := db.UserFollow{
			FollowerID: followerID,
			FolloweeID: followeeID,
		}

		if err := conn.Create(&follow).Error; err != nil {
			return err
		}

		return nil
	}

	// remote user
	// save follow request
	// check existing request
	tx := conn.Begin()
	var req *db.UserFollowRequest
	if err = tx.Where("follower_id = ? AND followee_id = ?", followerID, followeeID).First(&req).Error; err != nil {
		if !errors.Is(err, gorm.ErrRecordNotFound) {
			return err
		}

		req := db.UserFollowRequest{
			ID:         db.MustGenerateUUID(),
			FollowerID: followerID,
			FolloweeID: followeeID,
			Incoming:   false,
			URI:        sql.NullString{},
		}
		if err := s.conn.DB.Create(&req).Error; err != nil {
			return err
		}
	}

	if err := tx.Commit().Error; err != nil {
		return err
	}

	// send Follow to remote inbox
	reqID, err := s.idGetter.GetFollowRequestID(req)
	if err != nil {
		return err
	}
	if err := s.pubFollow.SendFollowRequest(reqID, follower, followee); err != nil {
		return err
	}

	return nil
}

func (s *DBUserFollowService) Unfollow(followerID db.UUID, followeeID db.UUID) error {
	conn := s.conn.DB

	err := conn.Delete(&db.UserFollow{}, "follower_id = ? AND followee_id = ?", followerID, followeeID).Error
	if err != nil {
		return err
	}

	return nil
}

package users

import (
	"context"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"gorm.io/gorm"
)

func IsFollowedBy(ctx context.Context, conn db.DBConn, followerID db.UUID, followeeID db.UUID) (bool, error) {
	var count int64
	err := conn.DB().Model(&db.UserFollow{}).Where("follower_id = ? AND followee_id = ?", followerID, followeeID).Count(&count).Error
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

func FindFollowers(ctx context.Context, conn db.DBConn, followeeID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	var (
		followers []FollowerInfo
		tx        *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.DB().Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("followee_id = ?", followeeID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.DB().Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("followee_id = ?", followeeID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC").Limit(limit)

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

func FindFollowing(ctx context.Context, conn db.DBConn, followerID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	var (
		followings []FollowerInfo
		tx         *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.DB().Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("follower_id = ?", followerID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.DB().Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("follower_id = ?", followerID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC").Limit(limit)

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

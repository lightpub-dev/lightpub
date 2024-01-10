package users

import (
	"context"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
)

func IsFollowedBy(ctx context.Context, conn db.DBConn, followerID string, followeeID string) (bool, error) {
	var count int
	err := conn.DB().GetContext(ctx, &count, "SELECT COUNT(*) FROM UserFollow WHERE follower_id=UUID_TO_BIN(?) AND followee_id=UUID_TO_BIN(?)", followerID, followeeID)
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

type FollowerInfo struct {
	ID          string `json:"id"`
	Username    string `json:"username"`
	Host        string `json:"host"`
	URL         string `json:"url"`
	Nickname    string `json:"nickname"`
	Bio         string `json:"bio"`
	IsFollowing bool   `json:"is_following"`
}

func CreateLocalUserURL(username string) string {
	return fmt.Sprintf("%s/user/%s", config.BaseURL, username)
}

func fillInLocalURL(follower *FollowerInfo) {
	if follower.URL == "" {
		follower.URL = CreateLocalUserURL(follower.Username)
	}
}

func FindFollowers(ctx context.Context, conn db.DBConn, followeeID string, viewerID string, beforeDate *time.Time, limit int64) ([]FollowerInfo, error) {
	var (
		followers []FollowerInfo
		sql       string
		params    []interface{}
	)
	if viewerID == "" {
		sql = `
SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,IFNULL(u.url, '') AS url,u.nickname,up.bio
FROM UserFollow uf
INNER JOIN User u ON uf.follower_id=u.id
INNER JOIN UserProfile up ON up.user_id=u.id
WHERE uf.followee_id=UUID_TO_BIN(?)
		`
		params = append(params, followeeID)
	} else {
		sql = `
SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,IFNULL(u.url, '') AS url,u.nickname,up.bio,
(
SELECT COUNT(*) FROM UserFollow uf2 WHERE uf2.follower_id=UUID_TO_BIN(?) AND uf2.followee_id=u.id
) AS is_following
FROM UserFollow uf
INNER JOIN User u ON uf.follower_id=u.id
INNER JOIN UserProfile up ON up.user_id=u.id
WHERE uf.followee_id=UUID_TO_BIN(?)
		`
		params = append(params, viewerID, followeeID)
	}

	if beforeDate != nil {
		sql += " AND up.created_at < ?"
		params = append(params, beforeDate)
	}

	sql += " ORDER BY up.created_at DESC LIMIT ?"
	params = append(params, limit)

	err := conn.DB().SelectContext(ctx, &followers, sql, params...)
	if err != nil {
		return nil, err
	}

	for i, follower := range followers {
		fillInLocalURL(&follower)
		followers[i] = follower
	}

	return followers, nil
}

func FindFollowing(ctx context.Context, conn db.DBConn, followerID string, viewerID string, beforeDate *time.Time, limit int64) ([]FollowerInfo, error) {
	var (
		following []FollowerInfo
		sql       string
		params    []interface{}
	)
	if viewerID == "" {
		sql = `
SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,IFNULL(u.url, '') AS url,u.nickname,up.bio
FROM UserFollow uf
INNER JOIN User u ON uf.followee_id=u.id
INNER JOIN UserProfile up ON up.user_id=u.id
WHERE uf.follower_id=UUID_TO_BIN(?)
		`
		params = append(params, followerID)
	} else {
		sql = `
SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,IFNULL(u.url, '') AS url,u.nickname,up.bio,
(
SELECT COUNT(*) FROM UserFollow uf2 WHERE uf2.follower_id=UUID_TO_BIN(?) AND uf2.followee_id=u.id
) AS is_following
FROM UserFollow uf
INNER JOIN User u ON uf.followee_id=u.id
INNER JOIN UserProfile up ON up.user_id=u.id
WHERE uf.follower_id=UUID_TO_BIN(?)
		`
		params = append(params, viewerID, followerID)
	}

	if beforeDate != nil {
		sql += " AND up.created_at < ?"
		params = append(params, beforeDate)
	}

	sql += " ORDER BY up.created_at DESC LIMIT ?"
	params = append(params, limit)

	err := conn.DB().SelectContext(ctx, &following, sql, params...)
	if err != nil {
		return nil, err
	}

	for i, follow := range following {
		fillInLocalURL(&follow)
		following[i] = follow
	}

	return following, nil
}

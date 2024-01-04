package users

import (
	"context"

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
	ID       string  `json:"id"`
	Username string  `json:"username"`
	Host     string  `json:"host"`
	URL      *string `json:"url"`
}

func FindFollowers(ctx context.Context, conn db.DBConn, followeeID string) ([]FollowerInfo, error) {
	var followers []FollowerInfo
	err := conn.DB().SelectContext(ctx, &followers, `
	SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,u.url
	FROM User u
	INNER JOIN UserFollow uf ON u.id=uf.follower_id
	WHERE uf.followee_id=UUID_TO_BIN(?)`, followeeID)
	if err != nil {
		return nil, err
	}

	return followers, nil
}

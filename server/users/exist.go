package users

import (
	"context"
	"errors"
	"strings"

	"github.com/google/uuid"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"gorm.io/gorm"
)

var (
	ErrInvalidUsername = errors.New("invalid username")
)

func ExistsByID(ctx context.Context, conn db.DBConn, userID string) (bool, error) {
	var count int64
	err := conn.DB().Model(&db.User{}).Where("id = ?", userID).Count(&count).Error
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

type parsedUsernameOrID struct {
	ID       db.UUID
	Username parsedUsername
}

type parsedUsername struct {
	Username string
	Host     string
}

func parseUsernameOrID(usernameOrID string) (parsedUsernameOrID, error) {
	// if str starts with @, then it's a username
	if strings.HasPrefix(usernameOrID, "@") {
		pu, err := parseUsername(usernameOrID[1:])
		if err != nil {
			return parsedUsernameOrID{}, err
		}
		return parsedUsernameOrID{Username: pu}, nil
	} else {
		// otherwise, it's an ID
		parsedID, err := uuid.Parse(usernameOrID)
		if err != nil {
			return parsedUsernameOrID{}, err
		}
		return parsedUsernameOrID{
			ID: db.UUID(parsedID),
		}, nil
	}
}

func parseUsername(username string) (parsedUsername, error) {
	parts := strings.Split(username, "@")
	if len(parts) == 1 {
		return parsedUsername{
			Username: parts[0],
			Host:     "",
		}, nil
	} else if len(parts) == 2 {
		realHost := parts[1]
		if realHost == config.MyHostname {
			realHost = ""
		}
		return parsedUsername{
			Username: parts[0],
			Host:     realHost,
		}, nil
	} else {
		return parsedUsername{}, ErrInvalidUsername
	}
}

func FindIDByUsername(ctx context.Context, conn db.DBConn, username string) (*db.User, error) {
	parsedUsernameOrID, err := parseUsernameOrID(username)
	if err != nil {
		return nil, err
	}

	var (
		user db.User
	)
	selectColumns := "id, username, host, nickname, url, inbox, outbox, (host IS NULL) AS is_local, bio"
	if parsedUsernameOrID.ID != (db.UUID{}) {
		// parsed ID
		parsedID := parsedUsernameOrID.ID
		err = conn.DB().Select(selectColumns).First(&user, "id = ?", parsedID).Error
	} else {
		// parsed Username
		parsedUsername := parsedUsernameOrID.Username
		if parsedUsername.Host == "" {
			// local user
			err = conn.DB().Select(selectColumns).First(&user, "username = ?", parsedUsername.Username).Error
		} else {
			// remote user
			err = conn.DB().Select(selectColumns).First(&user, "username = ? AND host = ?", parsedUsername.Username, parsedUsername.Host).Error
		}
	}

	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func CountLocalUsers(ctx context.Context, conn db.DBConn) (int64, error) {
	var count int64
	err := conn.DB().Model(&db.User{}).Where("host IS NULL").Count(&count).Error
	if err != nil {
		return 0, err
	}

	return count, nil
}

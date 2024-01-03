package users

import (
	"context"
	"database/sql"
	"errors"
	"strings"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

var (
	ErrInvalidUsername = errors.New("invalid username")
)

func ExistsByID(ctx context.Context, db db.DBOrTx, userID string) (bool, error) {
	var count int
	err := db.GetContext(ctx, &count, "SELECT COUNT(*) FROM User WHERE id=UUID_TO_BIN(?)", userID)
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

type parsedUsernameOrID struct {
	ID       string
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
		return parsedUsernameOrID{
			ID: usernameOrID,
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

func FindIDByUsername(ctx context.Context, db db.DBOrTx, username string) (*models.User, error) {
	parsedUsernameOrID, err := parseUsernameOrID(username)
	if err != nil {
		return nil, err
	}

	var stmt string
	params := make([]interface{}, 0, 2)
	if parsedUsernameOrID.ID != "" {
		// parsed ID
		parsedID := parsedUsernameOrID.ID
		stmt = "SELECT BIN_TO_UUID(id) AS id,username,host,nickname,url,inbox,outbox,is_local FROM User WHERE id=UUID_TO_BIN(?)"
		params = append(params, parsedID)
	} else {
		// parsed Username
		parsedUsername := parsedUsernameOrID.Username
		if parsedUsername.Host == "" {
			// local user
			stmt = "SELECT BIN_TO_UUID(id) AS id,username,host,nickname,url,inbox,outbox,is_local FROM User WHERE username=?"
			params = append(params, parsedUsername.Username)
		} else {
			// remote user
			stmt = "SELECT BIN_TO_UUID(id) AS id,username,host,nickname,url,inbox,outbox,is_local FROM User WHERE username=? AND host=?"
			params = append(params, parsedUsername.Username, parsedUsername.Host)
		}
	}

	var user models.User
	err = db.GetContext(ctx, &user, stmt, params...)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

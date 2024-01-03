package webfinger

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"strings"

	"github.com/jmoiron/sqlx"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/models"
)

var (
	ErrInvalidHost = errors.New("invalid host")
	ErrNotFound    = errors.New("not found")
)

func fetchUser(ctx context.Context, db *sqlx.DB, username string) (*models.User, error) {
	var user models.User
	err := db.GetContext(ctx, &user, "SELECT BIN_TO_UUID(id) AS id,username FROM User WHERE username = ?", username)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}
	return &user, nil
}

func createUserProfileURL(username string) string {
	return fmt.Sprintf("%s/user/@%s", config.MyHostname, username)
}

func createPersonURL(userID string) string {
	return fmt.Sprintf("%s/user/%s", config.MyHostname, userID)
}

func handleAcct(ctx context.Context, db *sqlx.DB, specifier string) (interface{}, error) {
	// split by @
	parts := strings.SplitN(specifier, "@", 2)

	var username string

	if len(parts) == 1 {
		username = parts[0]
	} else if len(parts) == 2 {
		// the latter part is the domain
		domain := parts[1]
		if domain != config.MyHostname {
			return nil, ErrInvalidHost
		}
		username = parts[0]
	} else {
		return nil, ErrBadFormat
	}

	user, err := fetchUser(ctx, db, username)
	if err != nil {
		return nil, err
	}

	if user == nil {
		return nil, ErrNotFound
	}

	response := acctResponse{
		Subject: fmt.Sprintf("acct:%s@%s", username, config.MyHostname),
		Links: []acctLink{
			{
				Rel:  "self",
				Ty:   "application/activity+json",
				Href: createPersonURL(username),
			},
			{
				Rel:  "http://webfinger.net/rel/profile-page",
				Ty:   "text/html",
				Href: createUserProfileURL(username),
			},
		},
	}

	return response, nil
}

type acctResponse struct {
	Subject string     `json:"subject"`
	Links   []acctLink `json:"links"`
}

type acctLink struct {
	Rel  string `json:"rel"`
	Ty   string `json:"type"`
	Href string `json:"href"`
}

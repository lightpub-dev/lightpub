package webfinger

import (
	"bytes"
	"context"
	"database/sql"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"text/template"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"gorm.io/gorm"
)

var (
	ErrInvalidHost    = errors.New("invalid host")
	ErrNotFound       = errors.New("not found")
	webfingerTemplate = template.Must(template.ParseFiles("templates/webfinger.xml"))
)

func fetchUser(ctx context.Context, conn *gorm.DB, username string) (*db.User, error) {
	var user db.User
	err := conn.WithContext(ctx).Find(&user, "username = ? AND host IS NULL", username).Error
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

func handleAcct(c echo.Context, conn *gorm.DB, specifier string) error {
	// split by @
	parts := strings.SplitN(specifier, "@", 2)

	var username string

	if len(parts) == 1 {
		username = parts[0]
	} else if len(parts) == 2 {
		// the latter part is the domain
		domain := parts[1]
		if domain != config.MyHostname {
			return c.String(http.StatusUnprocessableEntity, "invalid host")
		}
		username = parts[0]
	} else {
		return c.String(http.StatusBadRequest, "bad format")
	}

	user, err := fetchUser(c.Request().Context(), conn, username)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	if user == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	subject := fmt.Sprintf("acct:%s@%s", username, config.MyHostname)
	selfURL := createPersonURL(username)
	profileURL := createUserProfileURL(username)

	accept := c.Request().Header.Get("Accept")
	if strings.Contains(accept, "application/xrd+xml") {
		buf := new(bytes.Buffer)
		if err := webfingerTemplate.Execute(buf, map[string]string{
			"subject":    subject,
			"apiUrl":     selfURL,
			"profileUrl": profileURL,
		}); err != nil {
			return c.String(http.StatusInternalServerError, "internal server error")
		}
		return c.Blob(http.StatusOK, "application/xrd+xml", buf.Bytes())
	} else {
		response := acctResponse{
			Subject: subject,
			Links: []acctLink{
				{
					Rel:  "self",
					Ty:   "application/activity+json",
					Href: selfURL,
				},
				{
					Rel:  "http://webfinger.net/rel/profile-page",
					Ty:   "text/html",
					Href: profileURL,
				},
			},
		}
		return c.JSON(http.StatusOK, response)
	}
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

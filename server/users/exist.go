package users

import (
	"errors"
	"strings"

	"github.com/google/uuid"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"gorm.io/gorm"
)

var (
	ErrInvalidUsername = errors.New("invalid username")
	ErrInvalidUUID     = errors.New("invalid UUID")
)

type UserFinderService interface {
	ExistsByID(userID string) (bool, error)
	FindIDByUsername(username string) (*db.User, error)
	CountLocalUsers() (int64, error)
}

type DBUserFinderService struct {
	conn *db.DBConn
}

func ProvideDBUserFinder(conn *db.DBConn) *DBUserFinderService {
	return &DBUserFinderService{conn: conn}
}

func (f *DBUserFinderService) ExistsByID(userID string) (bool, error) {
	var count int64
	err := f.conn.DB.WithContext(f.conn.Ctx.Ctx).Model(&db.User{}).Where("id = ?", userID).Count(&count).Error
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
			return parsedUsernameOrID{}, ErrInvalidUUID
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

func (f *DBUserFinderService) FindIDByUsername(username string) (*db.User, error) {
	// ctx := f.conn.Ctx.Ctx
	conn := f.conn.DB

	username = strings.ReplaceAll(username, "%40", "@")

	parsedUsernameOrID, err := parseUsernameOrID(username)
	if err != nil {
		return nil, err
	}

	var (
		user db.User
	)
	selectColumns := "id, username, host, nickname, url, inbox, outbox, bio"
	if parsedUsernameOrID.ID != (db.UUID{}) {
		// parsed ID
		parsedID := parsedUsernameOrID.ID
		err = conn.Select(selectColumns).First(&user, "id = ?", parsedID).Error
	} else {
		// parsed Username
		parsedUsername := parsedUsernameOrID.Username
		if parsedUsername.Host == "" {
			// local user
			err = conn.Select(selectColumns).First(&user, "username = ?", parsedUsername.Username).Error
		} else {
			// remote user
			err = conn.Select(selectColumns).First(&user, "username = ? AND host = ?", parsedUsername.Username, parsedUsername.Host).Error
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

func (f *DBUserFinderService) CountLocalUsers() (int64, error) {
	var count int64
	err := f.conn.DB.WithContext(f.conn.Ctx.Ctx).Model(&db.User{}).Where("host IS NULL").Count(&count).Error
	if err != nil {
		return 0, err
	}

	return count, nil
}

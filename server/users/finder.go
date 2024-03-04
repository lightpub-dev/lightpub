package users

import (
	"errors"
	"fmt"
	"net/url"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
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
	FetchUserByID(userID db.UUID) (*db.User, error)
	FetchUserByURI(uri string) (*db.User, error)
	FetchUser(spec Specifier) (*db.User, error)
}

type DBUserFinderService struct {
	conn    db.DBConn
	pubUser *PubUserService
	id      pub.IDGetterService
}

func ProvideDBUserFinder(conn db.DBConn, pubUser *PubUserService, id pub.IDGetterService) *DBUserFinderService {
	return &DBUserFinderService{conn: conn, pubUser: pubUser, id: id}
}

func (f *DBUserFinderService) ExistsByID(userID string) (bool, error) {
	var count int64
	err := f.conn.DB.WithContext(f.conn.Ctx.Ctx).Model(&db.User{}).Where("id = ?", userID).Count(&count).Error
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

func (f *DBUserFinderService) FindIDByUsername(username string) (*db.User, error) {
	// ctx := f.conn.Ctx.Ctx
	conn := f.conn.DB

	var (
		user db.User
	)
	// local user
	err := conn.Model(&db.User{}).First(&user, "username = ? AND host IS NULL", username).Error

	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func (f *DBUserFinderService) FetchUserByID(userID db.UUID) (*db.User, error) {
	var user db.User
	err := f.conn.DB.Model(&db.User{}).First(&user, "id = ?", userID).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func (f *DBUserFinderService) FetchUserByURI(uri string) (*db.User, error) {
	url, err := url.Parse(uri)
	if err != nil {
		return nil, err
	}

	return f.FetchUser(NewSpecifierFromURI(url))
}

func (f *DBUserFinderService) fetchUserByUsernameAndHost(username string, host string) (*db.User, error) {
	// we can assume it is not a local user

	// check if the user is already in the local database
	var user *db.User
	if err := f.conn.DB.Model(&db.User{}).Where("username = ? AND host = ?", username, host).First(&user).Error; err != nil {
		if !errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, err
		}
	}

	// The user was found in the db and has remote info
	if user != nil && user.RemoteInfo != nil {
		// check if the user is up-to-date
		if isUpToDate(user.RemoteInfo.FetchedAt) {
			return user, nil
		}
	}

	// the user was not found or too old, fetch the user from the remote server
	return f.pubUser.FetchRemoteUserByUsername(username, host)
}

func isUpToDate(t time.Time) bool {
	return time.Since(t) < time.Hour*24
}

func (f *DBUserFinderService) fetchUserByURI(uri *url.URL) (*db.User, error) {
	// local user check
	localUserURI, err := f.id.ExtractLocalUserID(uri.String())
	if err != nil {
		return nil, err
	}
	if localUserURI != "" {
		var uuid db.UUID
		if err := db.ParseTo(&uuid, localUserURI); err != nil {
			return nil, fmt.Errorf("invalid local user URI: %w", err)
		}
		return f.FetchUserByID(uuid)
	}

	// check if the user is already in the local database
	var user *db.User
	if err := f.conn.DB.Model(&db.User{}).Where("uri = ?", uri.String()).Joins("RemoteInfo").First(&user).Error; err != nil {
		if !errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, err
		}
	}

	// The user was found in the db and has remote info
	if user != nil && user.RemoteInfo != nil {
		// check if the user is up-to-date
		if isUpToDate(user.RemoteInfo.FetchedAt) {
			return user, nil
		}
	}

	// the user was not found or too old, fetch the user from the remote server
	return f.pubUser.FetchRemoteUser(uri)
}

func (f *DBUserFinderService) FetchUser(specifier Specifier) (*db.User, error) {
	switch specifier.Type {
	case SpecifierTypeID:
		return f.FetchUserByID(specifier.ID)
	case SpecifierTypeUsernameAndHost:
		if specifier.UsernameAndHost.Host == "" || specifier.UsernameAndHost.Host == f.id.MyHostname() {
			// it is a local user, should be in the local database
			return f.FindIDByUsername(specifier.UsernameAndHost.Username)
		}

		// it is a remote user
		return f.fetchUserByUsernameAndHost(specifier.UsernameAndHost.Username, specifier.UsernameAndHost.Host)
	case SpecifierURI:
		return f.fetchUserByURI(specifier.URI)
	default:
		panic("unknown specifier type")
	}
}

func (f *DBUserFinderService) CountLocalUsers() (int64, error) {
	var count int64
	err := f.conn.DB.WithContext(f.conn.Ctx.Ctx).Model(&db.User{}).Where("host IS NULL").Count(&count).Error
	if err != nil {
		return 0, err
	}

	return count, nil
}

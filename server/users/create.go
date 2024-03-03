package users

import (
	"database/sql"
	"errors"
	"log"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/utils"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

var (
	ErrUsernameTaken = errors.New("username already taken")
)

type ErrorFieldNotSet struct {
	Field string
}

func newErrorFieldNotSet(field string) ErrorFieldNotSet {
	return ErrorFieldNotSet{Field: field}
}

func (e ErrorFieldNotSet) Error() string {
	return e.Field + " not set"
}

type UserCreateRequest struct {
	Username string `json:"username"`
	Nickname string `json:"nickname"`
	Password string `json:"password"`
}

type RemoteUserKey struct {
	ID           string
	PublicKeyPem string
}

type RemoteUser struct {
	ID                string
	Host              string
	PreferredUsername string
	Name              string
	Inbox             string
	Outbox            string
	SharedInbox       string
	Following         string
	Followers         string
	Liked             string
	Keys              []RemoteUserKey
}

type UserCreateService interface {
	CreateUser(u UserCreateRequest) error
	UpdateRemoteUser(u RemoteUser) (*db.User, error)
}

type DBUserCreateService struct {
	conn db.DBConn
	key  UserKeyService
}

func ProvideDBUserCreateService(conn db.DBConn, key UserKeyService) *DBUserCreateService {
	return &DBUserCreateService{conn, key}
}

func (s *DBUserCreateService) CreateUser(u UserCreateRequest) error {
	// check if username is taken
	tx := s.conn.DB.Begin()
	defer tx.Rollback()

	var count int64
	result := tx.Model(&db.User{}).Where("username = ?", u.Username).Count(&count)
	err := result.Error
	if err != nil {
		return err
	}

	if count > 0 {
		return ErrUsernameTaken
	}

	// hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(u.Password), bcrypt.DefaultCost)
	if err != nil {
		return err
	}

	userId, err := utils.GenerateUUID()
	if err != nil {
		return err
	}

	user := db.User{
		ID:       db.UUID(userId),
		Username: u.Username,
		Nickname: u.Nickname,
		Bpasswd:  sql.NullString{String: string(hashedPassword), Valid: true},
		Host:     sql.NullString{}, // null for local user
		Bio:      "",
	}

	result = tx.Create(&user)
	err = result.Error
	if err != nil {
		return err
	}

	// commit
	err = tx.Commit().Error
	if err != nil {
		return err
	}

	// generate key in background
	go s.key.GenerateKeyForUser(user.ID)

	return nil
}

func (s *DBUserCreateService) UpdateRemoteUser(u RemoteUser) (*db.User, error) {
	if u.PreferredUsername == "" {
		return nil, newErrorFieldNotSet("PreferredUsername")
	}

	tx := s.conn.DB.Begin()
	defer tx.Rollback()

	var userInDB db.User
	userInDBOk := true
	if err := tx.Where("uri = ?", u.ID).First(&userInDB).Error; err != nil {
		if !errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, err
		}
		userInDBOk = false
	}
	c := utils.ConvertToSqlString
	var user db.User
	log.Printf("userInDB: %v", userInDB)
	if userInDBOk {
		user = db.User{
			ID:          userInDB.ID,
			Username:    u.PreferredUsername,
			Nickname:    u.Name,
			URI:         sql.NullString{String: u.ID, Valid: true},
			Bpasswd:     sql.NullString{},
			Host:        sql.NullString{String: u.Host, Valid: true},
			Bio:         "",
			Inbox:       c(u.Inbox),
			Outbox:      c(u.Outbox),
			SharedInbox: c(u.SharedInbox),
			CreatedAt:   time.Now(),
		}
	} else {
		user = db.User{
			ID:          db.MustGenerateUUID(),
			Username:    u.PreferredUsername,
			Nickname:    u.Name,
			URI:         sql.NullString{String: u.ID, Valid: true},
			Bpasswd:     sql.NullString{},
			Host:        sql.NullString{String: u.Host, Valid: true},
			Bio:         "",
			Inbox:       c(u.Inbox),
			Outbox:      c(u.Outbox),
			SharedInbox: c(u.SharedInbox),
			CreatedAt:   time.Now(),
		}
	}
	remo := db.RemoteUser{
		UserID:    user.ID,
		Following: c(u.Following),
		Followers: c(u.Followers),
		Liked:     c(u.Liked),
		FetchedAt: time.Now(),
	}

	// save user and remote user
	if err := tx.Save(&user).Error; err != nil {
		return nil, err
	}
	if err := tx.Save(&remo).Error; err != nil {
		return nil, err
	}

	// delete all existing keys for this user
	if err := tx.Delete(&db.UserKey{}, "owner_id=?", user.ID).Error; err != nil {
		return nil, err
	}
	// append keys
	for _, key := range u.Keys {
		userKey := db.UserKey{
			ID:        key.ID,
			OwnerID:   user.ID,
			PublicKey: key.PublicKeyPem,
			UpdatedAt: time.Now(),
		}
		if err := tx.Create(&userKey).Error; err != nil {
			return nil, err
		}
	}

	// commit
	if err := tx.Commit().Error; err != nil {
		return nil, err
	}

	// generate key in background
	go s.key.GenerateKeyForUser(user.ID)

	return &user, nil
}

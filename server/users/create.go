package users

import (
	"database/sql"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/utils"
	"golang.org/x/crypto/bcrypt"
)

var (
	ErrUsernameTaken = errors.New("username already taken")
)

type UserCreateRequest struct {
	Username string `json:"username"`
	Nickname string `json:"nickname"`
	Password string `json:"password"`
}

type UserCreateService interface {
	CreateUser(u UserCreateRequest) error
}

type DBUserCreateService struct {
	conn db.DBConn
	key  UserKeyService
}

func ProvideDBUserCreateService(conn db.DBConn) *DBUserCreateService {
	return &DBUserCreateService{conn: conn}
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
		Bpasswd:  string(hashedPassword),
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

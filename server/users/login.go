package users

import (
	"errors"

	"github.com/google/uuid"
	"github.com/lightpub-dev/lightpub/db"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

var (
	ErrBadAuth      = errors.New("bad auth")
	ErrNotLoginable = errors.New("not local")
)

type AuthResult struct {
	UserID   db.UUID
	Username string
}

type UserLoginService interface {
	Login(username string, password string) (string, error)
	TokenAuth(token string) (*AuthResult, error)
}

type DBUserLoginService struct {
	conn db.DBConn
}

func ProvideDBUserLoginService(conn db.DBConn) *DBUserLoginService {
	return &DBUserLoginService{conn: conn}
}

func (s *DBUserLoginService) Login(username string, password string) (string, error) {
	var user db.User
	result := s.conn.DB.First(&user, "username = ? AND host IS NULL", username)
	err := result.Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return "", ErrBadAuth
		}
		return "", err
	}

	// check password
	if !user.Bpasswd.Valid {
		return "", ErrNotLoginable
	}
	if bcrypt.CompareHashAndPassword([]byte(user.Bpasswd.String), []byte(password)) != nil {
		return "", ErrBadAuth
	}

	// generate token
	token, err := generateToken()
	if err != nil {
		return "", err
	}

	// insert token
	result = s.conn.DB.Create(&db.UserToken{
		UserID: user.ID,
		Token:  token.String(),
	})
	err = result.Error
	if err != nil {
		return "", err
	}

	return token.String(), nil
}

func (s *DBUserLoginService) TokenAuth(token string) (*AuthResult, error) {
	var user db.UserToken
	result := s.conn.DB.Model(&db.UserToken{}).Where("token = ?", token).Joins("User").Where("User.host IS NULL").First(&user)
	// if not found, return 401
	if errors.Is(result.Error, gorm.ErrRecordNotFound) {
		return nil, nil
	}
	if result.Error != nil {
		return nil, result.Error
	}

	return &AuthResult{
		UserID:   user.UserID,
		Username: user.User.Username,
	}, nil
}

func generateToken() (uuid.UUID, error) {
	return uuid.NewRandom()
}

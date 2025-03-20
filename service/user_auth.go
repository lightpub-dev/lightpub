package service

import (
	"database/sql"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

var (
	ErrBadUserCreateParams   = NewServiceError(400, "bad user create params")
	ErrUsernameAlreadyExists = NewServiceError(400, "username already exists")
)

const (
	localDomain = "" // Empty string means local server
)

type UserCreateParams struct {
	Username string
	Nickname string
	Password string
}

func (s *ServiceState) CreateNewLocalUser(user UserCreateParams) (types.UserID, error) {
	userID := types.NewUserID()

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(user.Password), bcrypt.DefaultCost)
	if err != nil {
		return types.UserID{}, NewInternalServerErrorWithCause("failed to hash password", err)
	}
	hashedPasswordStr := string(hashedPassword)

	newUser := db.User{
		ID:               userID,
		Username:         user.Username,
		Domain:           localDomain,
		Nickname:         user.Nickname,
		Password:         sql.NullString{String: hashedPasswordStr, Valid: true},
		AutoFollowAccept: true,
	}
	if s.DB().Create(&newUser).Error != nil {
		return types.UserID{}, NewInternalServerErrorWithCause("failed to create user", err)
	}

	return userID, nil
}

func (s *ServiceState) LoginUser(username, password string) (*types.UserID, error) {
	var user db.User
	if err := s.DB().Where("username = ? AND domain = ? AND password IS NOT NULL", username, localDomain).First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.Password.String), []byte(password)); err != nil {
		if errors.Is(err, bcrypt.ErrMismatchedHashAndPassword) {
			return nil, nil
		}
		return nil, err
	}

	return &user.ID, nil
}

package service

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

var (
	ErrBadUserCreateParams   = NewServiceError(400, "bad user create params")
	ErrUsernameAlreadyExists = NewServiceError(400, "username already exists")
)

type UserCreateParams struct {
	Username string
	Nickname string
	Password string
}

func (s *ServiceState) CreateNewLocalUser(ctx context.Context, user UserCreateParams) (types.UserID, error) {
	userID := types.NewUserID()

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(user.Password), bcrypt.DefaultCost)
	if err != nil {
		return types.UserID{}, NewInternalServerErrorWithCause("failed to hash password", err)
	}
	hashedPasswordStr := string(hashedPassword)

	newUser := db.User{
		ID:               userID,
		Username:         user.Username,
		Domain:           types.EmptyDomain,
		Nickname:         user.Nickname,
		Password:         sql.NullString{String: hashedPasswordStr, Valid: true},
		AutoFollowAccept: true,
	}
	if s.DB(ctx).Create(&newUser).Error != nil {
		return types.UserID{}, NewInternalServerErrorWithCause("failed to create user", err)
	}

	return userID, nil
}

func (s *ServiceState) LoginUser(ctx context.Context, username, password string) (*types.UserID, error) {
	var user db.User
	if err := s.DB(ctx).Where("username = ? AND domain = '' AND password IS NOT NULL", username).First(&user).Error; err != nil {
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

// CheckUserLoginExpiration returns true if the user's login has not expired.
func (s *ServiceState) CheckUserLoginExpiration(ctx context.Context, userID types.UserID, loggedInAt time.Time) (bool, error) {
	user, err := s.FindUserByIDRaw(ctx, userID)
	if err != nil {
		return false, err
	}
	if user == nil {
		return false, nil
	}

	if user.AuthExpiredAt.Valid && loggedInAt.Before(user.AuthExpiredAt.Time) {
		return false, nil
	}

	return true, nil
}

func (s *ServiceState) SetUserLoginExpiration(ctx context.Context, userID types.UserID, expiresAt time.Time) error {
	return s.DB(ctx).Model(&db.User{}).Where("id = ?", userID).Update("auth_expired_at", expiresAt).Error
}

/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

package service

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"database/sql"
	"encoding/pem"
	"errors"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

const (
	privateKeyBits = 2048
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

func (s *State) CreateNewLocalUser(ctx context.Context, user UserCreateParams) (types.UserID, error) {
	userID := types.NewUserID()

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(user.Password), bcrypt.DefaultCost)
	if err != nil {
		return types.UserID{}, NewInternalServerErrorWithCause("failed to hash password", err)
	}
	hashedPasswordStr := string(hashedPassword)

	privateKey, err := rsa.GenerateKey(rand.Reader, privateKeyBits)
	if err != nil {
		return types.UserID{}, fmt.Errorf("failed to generate private key: %w", err)
	}
	publicKey := privateKey.PublicKey

	privateKeyBin, err := x509.MarshalPKCS8PrivateKey(privateKey)
	if err != nil {
		return types.UserID{}, fmt.Errorf("failed to marshal private key: %w", err)
	}
	publicKeyBin, err := x509.MarshalPKIXPublicKey(&publicKey)
	if err != nil {
		return types.UserID{}, fmt.Errorf("failed to marshal public key: %w", err)
	}

	publicKeyPem := pem.EncodeToMemory(
		&pem.Block{
			Type:  "PUBLIC KEY",
			Bytes: publicKeyBin,
		},
	)
	privateKeyPem := pem.EncodeToMemory(
		&pem.Block{
			Type:  "PRIVATE KEY",
			Bytes: privateKeyBin,
		},
	)

	newUser := models.User{
		ID:               userID,
		Username:         user.Username,
		Domain:           types.EmptyDomain,
		Nickname:         user.Nickname,
		Password:         sql.NullString{String: hashedPasswordStr, Valid: true},
		AutoFollowAccept: true,
		PublicKey:        stringToSql(string(publicKeyPem)),
		PrivateKey:       stringToSql(string(privateKeyPem)),
	}
	if err := s.DB(ctx).Create(&newUser).Error; err != nil {
		if errors.Is(err, gorm.ErrDuplicatedKey) {
			return types.UserID{}, ErrUsernameAlreadyExists
		}
		return types.UserID{}, NewInternalServerErrorWithCause("failed to create user", err)
	}

	return userID, nil
}

func (s *State) LoginUser(ctx context.Context, username, password string) (*types.UserID, error) {
	var user models.User
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

func (s *State) LogoutAllUser(ctx context.Context, userID types.UserID) error {
	return s.DB(ctx).Model(&models.User{}).Where("id = ?", userID).Update("auth_expired_at", time.Now()).Error
}

// CheckUserLoginExpiration returns true if the user's login has not expired.
func (s *State) CheckUserLoginExpiration(ctx context.Context, userID types.UserID, loggedInAt time.Time) (bool, error) {
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

func (s *State) SetUserLoginExpiration(ctx context.Context, userID types.UserID, expiresAt time.Time) error {
	return s.DB(ctx).Model(&models.User{}).Where("id = ?", userID).Update("auth_expired_at", expiresAt).Error
}

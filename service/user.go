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
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

func (s *State) FindUserByIDRaw(ctx context.Context, id types.UserID) (*db.User, error) {
	var user db.User
	if err := s.DB(ctx).Where("id = ?", id).First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func (s *State) makeSimpleUserFromDB(user *db.User) types.SimpleUser {
	cleanBio := bioSanitizer.Sanitize(user.Bio)

	return types.SimpleUser{
		ID:       user.ID,
		Username: user.Username,
		Domain:   user.Domain,
		Nickname: user.Nickname,
		Bio:      cleanBio,
		Avatar:   user.Avatar,
	}
}

func (s *State) FindUserByID(ctx context.Context, id types.UserID) (*types.SimpleUser, error) {
	user, err := s.FindUserByIDRaw(ctx, id)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, nil
	}

	u := s.makeSimpleUserFromDB(user)
	return &u, nil
}

func (s *State) FindLocalUserIDBySpecifier(ctx context.Context, specifier *types.UserSpecifier) (*types.UserID, error) {
	switch specifier.Kind {
	case types.UserSpecifierID:
		return &specifier.ID, nil
	case types.UserSpecifierUsername:
		var user db.User
		if err := s.DB(ctx).Where("username = ? AND domain = ?", specifier.Username.Username, specifier.Username.Domain).First(&user).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return nil, nil
			}
			return nil, err
		}
		return &user.ID, nil
	case types.UserSpecifierURL:
		localUserID, ok := s.extractUserIDFromLocalURL(specifier.URL)
		if !ok {
			return nil, nil
		}
		return &localUserID, nil
	}

	panic("unreachable")
}

func (s *State) FindUserIDBySpecifierWithRemote(ctx context.Context, specifier *types.UserSpecifier) (*types.UserID, error) {
	// try local
	localUserID, err := s.FindLocalUserIDBySpecifier(ctx, specifier)
	if err != nil {
		return nil, err
	}
	if localUserID != nil {
		return localUserID, nil
	}

	// try remote
	// TODO: implement remote user lookup

	return nil, nil
}

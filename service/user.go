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
	"fmt"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

func (s *State) FindUserByIDRaw(ctx context.Context, id types.UserID) (*models.User, error) {
	var user models.User
	if err := s.DB(ctx).Where("id = ?", id).First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func (s *State) makeSimpleUserFromDB(user *models.User) types.SimpleUser {
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

func (s *State) FindApubUserByID(ctx context.Context, id types.UserID) (*types.ApubUser, error) {
	user, err := s.FindUserByIDRaw(ctx, id)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, nil
	}

	u := s.makeSimpleUserFromDB(user)

	var (
		pubkey  string
		privkey string
		keyID   string
	)
	if u.Domain == types.EmptyDomain {
		pubkey = user.PublicKey.String
		privkey = user.PrivateKey.String
		keyID = s.keyIDForLocalUser(user.ID)
	}

	var (
		inbox       string
		sharedInbox string
		following   string
		followers   string
		url         string
		viewURL     string
	)
	if u.Domain != types.EmptyDomain {
		inbox = user.Inbox.String
		sharedInbox = user.SharedInbox.String
		following = user.FollowingURL.String
		followers = user.FollowersURL.String
		url = user.URL.String
		viewURL = user.ViewURL.String
	} else {
		inbox = s.inboxForLocalUser(user.ID)
		sharedInbox = s.sharedInboxForLocalUser()
		col := s.collectionURLsForLocalUser(user.ID)
		following = col.Following
		followers = col.Followers
		url = s.urlForLocalUser(user.ID)
		viewURL = s.viewURLForLocalUser(user.ID)
	}

	a := types.ApubUserData{
		PublicKey_:  pubkey,
		PrivateKey_: privkey,
		KeyID_:      keyID,

		Bio: s.renderBio(user.Bio),

		URL:     url,
		ViewURL: viewURL,

		Following:   following,
		Followers:   followers,
		Inbox:       inbox,
		SharedInbox: sharedInbox,
	}

	return &types.ApubUser{
		Basic: u,
		Apub:  a,
	}, nil
}

func (s *State) findApubURLForUserID(ctx context.Context, id types.UserID) (string, error) {
	user, err := s.FindUserByIDRaw(ctx, id)
	if err != nil {
		return "", fmt.Errorf("error fetching user %s: %w", id, err)
	}
	if user == nil {
		return "", nil
	}

	if user.URL.Valid {
		return user.URL.String, nil
	}
	return s.urlForLocalUser(user.ID), nil
}

func (s *State) FindLocalUserIDBySpecifier(ctx context.Context, specifier *types.UserSpecifier) (*types.UserID, error) {
	switch specifier.Kind {
	case types.UserSpecifierID:
		return &specifier.ID, nil
	case types.UserSpecifierUsername:
		var user models.User
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

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
	"database/sql"
	"errors"
	"fmt"
	"net/url"
	"time"

	"github.com/lightpub-dev/lightpub/apub"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

func (s *State) fetchAndStoreRemoteUser(ctx context.Context, specifier *types.UserSpecifier) (types.UserID, error) {
	actor, err := s.delivery.FetchRemoteUserBySpecifier(ctx, specifier)
	if err != nil {
		return types.UserID{}, err
	}

	userID, err := s.upsertActor(ctx, actor)
	if err != nil {
		return types.UserID{}, err
	}
	return userID, nil
}

func (s *State) upsertActor(ctx context.Context, actor *apub.User) (types.UserID, error) {
	if s.inTx {
		return s.upsertActorInTx(ctx, actor)
	} else {
		var userID types.UserID
		err := s.WithTransaction(func(tx *State) error {
			u, err := tx.upsertActorInTx(ctx, actor)
			userID = u
			return err
		})
		return userID, err
	}
}

func (s *State) upsertActorInTx(ctx context.Context, actor *apub.User) (types.UserID, error) {
	var user models.User
	// try to fetch with URL
	found := true
	if err := s.DB(ctx).Where("url = ?", actor.ID).First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			found = false
		} else {
			return types.UserID{}, err
		}
	}

	// update user model
	isUpdate := found
	if err := setDBUserFromActor(actor, &user, isUpdate); err != nil {
		return types.UserID{}, err
	}

	if err := s.DB(ctx).Save(&user).Error; err != nil {
		return types.UserID{}, err
	}

	// replace public key
	if err := s.DB(ctx).Where("owner_id", user.ID).Delete(&models.RemotePublicKey{}).Error; err != nil {
		return types.UserID{}, err
	}
	if err := s.DB(ctx).Create(&models.RemotePublicKey{
		OwnerID:   user.ID,
		PublicKey: actor.PublicKey.PublicKeyPem,
		KeyID:     actor.PublicKey.ID,
	}).Error; err != nil {
		return types.UserID{}, err
	}

	return user.ID, nil
}

func setDBUserFromActor(actor *apub.User, user *models.User, update bool) error {
	actorID, err := url.Parse(actor.ID)
	if err != nil {
		return fmt.Errorf("invalid actor ID: %w", err)
	}

	user.Username = actor.PreferredUsername
	user.Domain = actorID.Host
	user.Password = sql.NullString{} // no auth
	user.Nickname = actor.Name
	user.Bio = actor.Summary
	// TODO: user.Avatar
	user.URL = stringToNullableSql(actor.ID)
	user.Inbox = stringToNullableSql(actor.Inbox)
	user.SharedInbox = stringToNullableSql(actor.Endpoints.SharedInbox)
	user.Outbox = stringToNullableSql(actor.Outbox)
	user.PrivateKey = sql.NullString{} // no signing
	user.PublicKey = sql.NullString{}  // no signing
	user.CreatedAt = sql.NullTime{}
	user.FetchedAt = sql.NullTime{Valid: true, Time: time.Now().UTC()}
	user.ViewURL = stringToNullableSql(actor.URL)
	user.FollowingURL = stringToNullableSql(actor.Following)
	user.FollowersURL = stringToNullableSql(actor.Followers)
	user.AutoFollowAccept = !actor.ManuallyApprovesFollowers
	user.AuthExpiredAt = sql.NullTime{}
	user.IsBot = actor.Kind != "Person"
	user.IsAdmin = false // remote user can never be admin
	user.HideFollows = false

	return nil
}

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
	"fmt"

	"github.com/lightpub-dev/lightpub/apub"
	"github.com/lightpub-dev/lightpub/types"
)

type followObjectPair struct {
	ActorID  types.UserID
	ObjectID types.UserID
}

func (s *State) parseFollowActivity(ctx context.Context, activity apub.FollowActivity) (followObjectPair, error) {
	actorURL, err := types.NewUserURLFromString(activity.Actor)
	if err != nil {
		return followObjectPair{}, err
	}
	actorID, err := s.FindUserIDBySpecifierWithRemote(ctx, actorURL)
	if err != nil {
		return followObjectPair{}, fmt.Errorf("failed to find actor: %w", err)
	}
	if actorID == nil {
		return followObjectPair{}, ErrFollowerNotFound
	}

	objectURL, err := types.NewUserURLFromString(activity.Object.ID)
	if err != nil {
		return followObjectPair{}, err
	}
	objectID, err := s.FindUserIDBySpecifierWithRemote(ctx, objectURL)
	if err != nil {
		return followObjectPair{}, fmt.Errorf("failed to find object: %w", err)
	}
	if objectID == nil {
		return followObjectPair{}, ErrFolloweeNotFound
	}

	return followObjectPair{
		ActorID:  *actorID,
		ObjectID: *objectID,
	}, nil
}

func (s *State) handleFollowActivity(ctx context.Context, activity apub.FollowActivity) error {
	pair, err := s.parseFollowActivity(ctx, activity)
	if err != nil {
		return err
	}

	if err := s.FollowUser(ctx, pair.ActorID, pair.ObjectID); err != nil {
		return fmt.Errorf("failed to follow: %w", err)
	}

	return nil
}

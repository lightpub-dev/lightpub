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
)

func (s *State) handleUndoFollowActivity(ctx context.Context, activity apub.UndoActivity) error {
	pair, err := s.parseFollowActivity(ctx, *activity.Object.Follow)
	if err != nil {
		return err
	}

	if err := s.UnfollowUser(ctx, pair.ActorID, pair.ObjectID); err != nil {
		return fmt.Errorf("failed to unfollow: %w", err)
	}

	return nil
}

func (s *State) handleUndoActivity(ctx context.Context, activity apub.UndoActivity) error {
	switch activity.Object.Kind {
	case apub.UndoableActivityTypeFollow:
		return s.handleUndoActivity(ctx, activity)
	}

	return fmt.Errorf("unsupported undo activity type: %s", activity.Object.Kind)
}

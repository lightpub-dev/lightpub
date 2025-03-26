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

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var (
	ErrBlockerNotFound = NewServiceError(404, "blocker not found")
	ErrBlockedNotFound = NewServiceError(404, "blocked not found")
)

func (s *State) BlockUser(ctx context.Context, blockerID types.UserID, blockedID types.UserID) error {
	blockerUser, err := s.FindUserByID(ctx, blockerID)
	if err != nil {
		return err
	}
	if blockerUser == nil {
		return ErrBlockerNotFound
	}

	blockedUser, err := s.FindUserByID(ctx, blockedID)
	if err != nil {
		return err
	}
	if blockedUser == nil {
		return ErrBlockedNotFound
	}

	result := s.DB(ctx).Clauses(clause.OnConflict{DoNothing: true}).Create(&models.UserBlock{
		BlockerID: blockerID,
		BlockedID: blockedID,
	})
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to create block", err)
	}
	if result.RowsAffected == 0 {
		// block already exists
		return nil
	}

	if blockerUser.IsLocal() && blockedUser.IsRemote() {
		// TODO: apub Block?
	}

	// unfollow each other
	err = s.UnfollowUser(ctx, blockerID, blockedID)
	if err != nil {
		return err
	}
	err = s.UnfollowUser(ctx, blockedID, blockerID)
	if err != nil {
		return err
	}

	return nil
}

func (s *State) UnblockUser(ctx context.Context, blockerID types.UserID, blockedID types.UserID) error {
	blockerUser, err := s.FindUserByID(ctx, blockerID)
	if err != nil {
		return err
	}
	if blockerUser == nil {
		return ErrBlockerNotFound
	}

	blockedUser, err := s.FindUserByID(ctx, blockedID)
	if err != nil {
		return err
	}
	if blockedUser == nil {
		return ErrBlockedNotFound
	}

	result := s.DB(ctx).Where(&models.UserBlock{
		BlockerID: blockerID,
		BlockedID: blockedID,
	}).Delete(&models.UserBlock{})
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to delete block", result.Error)
	}
	if result.RowsAffected == 0 {
		return nil
	}

	if blockerUser.IsLocal() && blockedUser.IsRemote() {
		// TODO: apub Undo Block?
	}

	return nil
}

func (s *State) IsBlocking(ctx context.Context, blockerID types.UserID, blockedID types.UserID) (bool, error) {
	var block models.UserBlock
	result := s.DB(ctx).Where(&models.UserBlock{
		BlockerID: blockerID,
		BlockedID: blockedID,
	}).First(&block)

	if result.Error != nil {
		if errors.Is(result.Error, gorm.ErrRecordNotFound) {
			return false, nil
		}
		return false, NewInternalServerErrorWithCause("failed to check block status", result.Error)
	}

	return true, nil
}

func (s *State) IsBlockingOrBlocked(ctx context.Context, user1 types.UserID, user2 types.UserID) (bool, error) {
	var count int64
	result := s.DB(ctx).Model(&models.UserBlock{}).Where(
		"(blocker_id = ? AND blocked_id = ?) OR (blocker_id = ? AND blocked_id = ?)",
		user1, user2, user2, user1,
	).Count(&count)

	if result.Error != nil {
		return false, NewInternalServerErrorWithCause("failed to check block status", result.Error)
	}

	return count > 0, nil
}

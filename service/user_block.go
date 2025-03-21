package service

import (
	"context"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
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

	result := s.DB(ctx).Clauses(clause.OnConflict{DoNothing: true}).Create(&db.UserBlock{
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

	result := s.DB(ctx).Where(&db.UserBlock{
		BlockerID: blockerID,
		BlockedID: blockedID,
	}).Delete(&db.UserBlock{})
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
	var block db.UserBlock
	result := s.DB(ctx).Where(&db.UserBlock{
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
	result := s.DB(ctx).Model(&db.UserBlock{}).Where(
		"(blocker_id = ? AND blocked_id = ?) OR (blocker_id = ? AND blocked_id = ?)",
		user1, user2, user2, user1,
	).Count(&count)

	if result.Error != nil {
		return false, NewInternalServerErrorWithCause("failed to check block status", result.Error)
	}

	return count > 0, nil
}

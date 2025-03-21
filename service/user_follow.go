package service

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var (
	ErrCannotFollowSelf  = NewServiceError(400, "cannot follow self")
	ErrCannotFollowBlock = NewServiceError(400, "cannot follow blocked user")
	ErrFollowerNotFound  = NewServiceError(404, "follower not found")
	ErrFolloweeNotFound  = NewServiceError(404, "followee not found")

	FollowStateNo      FollowState = 0
	FollowStateYes     FollowState = 1
	FollowStatePending FollowState = 2
)

type FollowState = int

func (s *State) FollowUser(
	ctx context.Context,
	followerID types.UserID,
	followeeID types.UserID,
) error {
	// Self check
	if followerID == followeeID {
		return ErrCannotFollowSelf
	}

	followerUser, err := s.FindUserByID(ctx, followerID)
	if err != nil {
		return err
	}
	if followerUser == nil {
		return ErrFollowerNotFound
	}

	followeeUser, err := s.FindUserByIDRaw(ctx, followeeID)
	if err != nil {
		return err
	}
	if followeeUser == nil {
		return ErrFolloweeNotFound
	}
	autoAcceptFollow := followeeUser.AutoFollowAccept

	// block check
	blocked, err := s.IsBlockingOrBlocked(ctx, followerID, followeeID)
	if err != nil {
		return err
	}
	if blocked {
		return ErrCannotFollowBlock
	}

	follow := db.UserFollow{
		FollowerID: followerID,
		FollowedID: followeeID,
		Pending:    !autoAcceptFollow,
	}
	result := s.DB(ctx).Clauses(
		clause.OnConflict{
			DoNothing: true,
		},
	).Create(&follow)
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to create follow", err)
	}
	if result.RowsAffected == 0 {
		// follow already exists
		return nil
	}

	if followerUser.IsLocal() && followeeUser.Domain != types.EmptyDomain {
		// TODO: send apub Follow
	}

	return nil
}

func (s *State) UnfollowUser(
	ctx context.Context,
	followerID types.UserID,
	followeeID types.UserID,
) error {
	if followerID == followeeID {
		return ErrCannotFollowSelf
	}

	followerUser, err := s.FindUserByID(ctx, followerID)
	if err != nil {
		return err
	}
	if followerUser == nil {
		return ErrFollowerNotFound
	}

	followeeUser, err := s.FindUserByID(ctx, followeeID)
	if err != nil {
		return err
	}
	if followeeUser == nil {
		return ErrFolloweeNotFound
	}

	result := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		followerID, followeeID,
	).Delete(&db.UserFollow{})
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to delete follow", err)
	}
	if result.RowsAffected == 0 {
		// follow does not exist
		return nil
	}

	if followerUser.IsLocal() && followeeUser.IsRemote() {
		// TODO: send apub Undo(Follow)
	}

	return nil
}

func (s *State) RejectFollowUser(
	ctx context.Context,
	rejectorID types.UserID,
	rejectedID types.UserID,
) error {
	if rejectorID == rejectedID {
		return ErrCannotFollowSelf
	}

	rejectorUser, err := s.FindUserByID(ctx, rejectorID)
	if err != nil {
		return err
	}
	if rejectorUser == nil {
		return ErrFolloweeNotFound
	}

	rejectedUser, err := s.FindUserByID(ctx, rejectedID)
	if err != nil {
		return err
	}
	if rejectedUser == nil {
		return ErrFollowerNotFound
	}

	result := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		rejectedID, rejectorID,
	).Delete(&db.UserFollow{})
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to reject follow", err)
	}
	if result.RowsAffected == 0 {
		// follow does not exist
		return nil
	}

	if rejectorUser.IsLocal() && rejectedUser.IsRemote() {
		// TODO: send apub Reject(Follow)
	}

	return nil
}

func (s *State) AcceptFollow(
	ctx context.Context,
	acceptorID types.UserID,
	accepteeID types.UserID,
) error {
	if acceptorID == accepteeID {
		return ErrCannotFollowSelf
	}

	acceptorUser, err := s.FindUserByID(ctx, acceptorID)
	if err != nil {
		return err
	}
	if acceptorUser == nil {
		return ErrFolloweeNotFound
	}

	accepteeUser, err := s.FindUserByID(ctx, accepteeID)
	if err != nil {
		return err
	}
	if accepteeUser == nil {
		return ErrFollowerNotFound
	}

	// block check
	blocked, err := s.IsBlockingOrBlocked(ctx, acceptorID, accepteeID)
	if err != nil {
		return err
	}
	if blocked {
		return ErrCannotFollowBlock
	}

	result := s.DB(ctx).Model(&db.UserFollow{}).Where(
		"follower_id = ? AND followed_id = ? AND pending = true",
		accepteeID, acceptorID,
	).Update("pending", false)
	if result.Error != nil {
		return NewInternalServerErrorWithCause("failed to accept follow", err)
	}
	if result.RowsAffected == 0 {
		// follow does not exist or is not pending
		return nil
	}

	if acceptorUser.IsLocal() && accepteeUser.IsRemote() {
		// TODO: send apub Accept(Follow)
	}

	return nil
}

func (s *State) GetFollowState(
	ctx context.Context,
	followerID types.UserID, followeeID types.UserID,
) (FollowState, error) {
	if followerID == followeeID {
		return FollowStateNo, ErrCannotFollowSelf
	}

	var follow db.UserFollow
	if err := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		followerID, followeeID,
	).First(&follow).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return FollowStateNo, nil
		}
		return FollowStateNo, err
	}

	if follow.Pending {
		return FollowStatePending, nil
	}

	return FollowStateYes, nil
}

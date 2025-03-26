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
	"log/slog"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/service/notification"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var (
	ErrCannotFollowSelf  = NewServiceError(400, "cannot follow self")
	ErrCannotFollowBlock = NewServiceError(400, "cannot follow blocked user")
	ErrFollowerNotFound  = NewServiceError(404, "follower not found")
	ErrFolloweeNotFound  = NewServiceError(404, "followee not found")
)

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

	follow := models.UserFollow{
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

	// send notification if followee is local
	if followeeUser.Domain == types.EmptyDomain {
		var n notification.Body
		if autoAcceptFollow {
			n = &notification.Followed{
				FollowerUserID: followerID,
			}
		} else {
			n = &notification.FollowRequested{
				RequesterUserID: followerID,
			}
		}
		if err := s.AddNotification(ctx, followeeID, n); err != nil {
			slog.ErrorContext(ctx, "failed to send notification", "followerID", followerID, "followeeID", followeeID, "err", err)
		}
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
	).Delete(&models.UserFollow{})
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
	).Delete(&models.UserFollow{})
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

	result := s.DB(ctx).Model(&models.UserFollow{}).Where(
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

	// send notification if acceptee is local
	if accepteeUser.IsLocal() {
		n := &notification.FollowAccepted{
			AcceptorUserID: acceptorID,
		}
		if err := s.AddNotification(ctx, accepteeID, n); err != nil {
			slog.ErrorContext(ctx, "failed to send notification", "acceptorID", acceptorID, "accepteeID", accepteeID, "err", err)
		}
	}

	if acceptorUser.IsLocal() && accepteeUser.IsRemote() {
		// TODO: send apub Accept(Follow)
	}

	return nil
}

func (s *State) GetFollowState(
	ctx context.Context,
	followerID types.UserID, followeeID types.UserID,
) (types.FollowState, error) {
	if followerID == followeeID {
		return types.FollowStateNo, ErrCannotFollowSelf
	}

	var follow models.UserFollow
	if err := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		followerID, followeeID,
	).First(&follow).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return types.FollowStateNo, nil
		}
		return types.FollowStateNo, err
	}

	if follow.Pending {
		return types.FollowStatePending, nil
	}

	return types.FollowStateYes, nil
}

func (s *State) GetUserFollowingList(
	ctx context.Context,
	userID types.UserID,
	limit int,
	page int,
) ([]types.SimpleUser, error) {
	var follows []models.ActualUserFollow
	if err := s.DB(ctx).Model(&models.ActualUserFollow{}).Where(
		"follower_id = ?", userID,
	).Order("created_at DESC").Joins("Followed").Limit(limit).Offset(limit * page).Find(&follows).Error; err != nil {
		return nil, err
	}

	followings := make([]types.SimpleUser, 0, len(follows))
	for _, follow := range follows {
		followings = append(followings, s.makeSimpleUserFromDB(&follow.Followed))
	}

	return followings, nil
}

func (s *State) GetUserFollowersList(
	ctx context.Context,
	userID types.UserID,
	limit int,
	page int,
) ([]types.SimpleUser, error) {
	var follows []models.ActualUserFollow
	if err := s.DB(ctx).Model(&models.ActualUserFollow{}).Where(
		"followed_id = ?", userID,
	).Order("created_at DESC").Joins("Follower").Limit(limit).Offset(limit * page).Find(&follows).Error; err != nil {
		return nil, err
	}

	followers := make([]types.SimpleUser, 0, len(follows))
	for _, follow := range follows {
		followers = append(followers, s.makeSimpleUserFromDB(&follow.Follower))
	}

	return followers, nil
}

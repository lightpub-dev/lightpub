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
	"log/slog"
	"strconv"

	"github.com/lightpub-dev/lightpub/apub"
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

type followActivityData struct {
	Activity any

	Follower types.ApubUser
	Followee types.ApubUser
}

func (s *State) createLocalFollowObject(ctx context.Context, followID int, followerID types.UserID, followeeID types.UserID) (followActivityData, error) {
	followURL := s.BaseURL().JoinPath("follow", strconv.Itoa(followID)).String()

	follower, err := s.FindApubUserByID(ctx, followerID)
	if err != nil {
		return followActivityData{}, err
	}
	if follower == nil {
		return followActivityData{}, fmt.Errorf("follower user not found: %s", followerID)
	}
	followee, err := s.FindApubUserByID(ctx, followeeID)
	if err != nil {
		return followActivityData{}, err
	}
	if followee == nil {
		return followActivityData{}, fmt.Errorf("followee user not found: %s", followeeID)
	}

	activity := apub.NewFollowActivity(
		followURL,
		follower.Apub.URL,
		followee.Apub.URL,
	)
	return followActivityData{
		Activity: activity,
		Follower: *follower,
		Followee: *followee,
	}, nil
}

func (s *State) createRemoteFollowObject(ctx context.Context, followURL string, followerID types.UserID, followeeID types.UserID) (followActivityData, error) {
	follower, err := s.FindApubUserByID(ctx, followerID)
	if err != nil {
		return followActivityData{}, err
	}
	if follower == nil {
		return followActivityData{}, fmt.Errorf("follower user not found: %s", followerID)
	}
	followee, err := s.FindApubUserByID(ctx, followeeID)
	if err != nil {
		return followActivityData{}, err
	}
	if followee == nil {
		return followActivityData{}, fmt.Errorf("followee user not found: %s", followeeID)
	}

	activity := apub.NewFollowActivity(
		followURL,
		follower.Apub.URL,
		followee.Apub.URL,
	)
	return followActivityData{
		Activity: activity,
		Follower: *follower,
		Followee: *followee,
	}, nil
}

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
		activity, err := s.createLocalFollowObject(ctx, follow.ID, followerID, followeeID)
		if err != nil {
			return fmt.Errorf("failed to create follow activity: %w", err)
		}
		if err := s.delivery.QueueActivity(ctx, activity.Activity, activity.Follower, []string{
			activity.Followee.Apub.PreferredInbox(),
		}); err != nil {
			return fmt.Errorf("failed to queue follow activity: %w", err)
		}
	}

	return nil
}

func (s *State) UnfollowUser(
	ctx context.Context,
	followerID types.UserID,
	followeeID types.UserID,
) error {
	return s.WithTransaction(func(tx *State) error {
		return tx.unfollowUserInTx(ctx, followerID, followeeID)
	})
}

func (s *State) unfollowUserInTx(
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

	// find follow
	var follow models.UserFollow
	if err := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		followerID, followeeID,
	).Clauses(
		clause.Locking{Strength: clause.LockingStrengthUpdate},
	).First(&follow).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// follow does not exist
			return nil
		}
		return fmt.Errorf("failed to find follow: %w", err)
	}

	// delete it
	if err := s.DB(ctx).Unscoped().Delete(&follow).Error; err != nil {
		return fmt.Errorf("failed to delete follow: %w", err)
	}

	if followerUser.IsLocal() && followeeUser.IsRemote() {
		// send apub Undo(Follow)
		followActivity, err := s.createLocalFollowObject(ctx, follow.ID, followerID, followeeID)
		if err != nil {
			return fmt.Errorf("failed to create follow activity: %w", err)
		}
		undoActivity := apub.NewUndoActivity(followActivity.Follower.Apub.URL, followActivity.Activity.(apub.FollowActivity).AsUndoable())
		if err := s.delivery.QueueActivity(ctx, undoActivity, followActivity.Follower, []string{
			followActivity.Followee.Apub.PreferredInbox(),
		}); err != nil {
			return fmt.Errorf("failed to queue undo activity: %w", err)
		}
	}

	return nil
}

func (s *State) RejectFollowUser(
	ctx context.Context,
	rejectorID types.UserID,
	rejectedID types.UserID,
) error {
	return s.WithTransaction(func(tx *State) error {
		return tx.rejectFollowUserInTx(ctx, rejectorID, rejectedID)
	})
}

func (s *State) rejectFollowUserInTx(
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

	// get follow
	var follow models.UserFollow
	if err := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		rejectedID, rejectorID,
	).Clauses(clause.Locking{Strength: clause.LockingStrengthUpdate}).First(&follow).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// follow does not exist
			return nil
		}
		return fmt.Errorf("failed to find follow: %w", err)
	}

	// delete it
	if err := s.DB(ctx).Unscoped().Delete(&follow).Error; err != nil {
		return fmt.Errorf("failed to delete follow: %w", err)
	}

	if rejectorUser.IsLocal() && rejectedUser.IsRemote() {
		// send apub Reject(Follow)
		rejectID := s.urlForLocalFollowReject(follow.ID)
		followActivity, err := s.createRemoteFollowObject(ctx, follow.URL.String, rejectedID, rejectorID)
		if err != nil {
			return fmt.Errorf("failed to create follow activity: %w", err)
		}
		rejectActivitty := apub.NewRejectActivityWithID(rejectID, followActivity.Follower.Apub.URL, followActivity.Activity.(apub.FollowActivity).AsRejectable())
		if err := s.delivery.QueueActivity(ctx, rejectActivitty, followActivity.Followee, []string{
			followActivity.Follower.Apub.PreferredInbox(),
		}); err != nil {
			return fmt.Errorf("failed to queue undo activity: %w", err)
		}
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

	// get follow
	var follow models.UserFollow
	if err := s.DB(ctx).Where(
		"follower_id = ? AND followed_id = ?",
		accepteeID, acceptorID,
	).Clauses(clause.Locking{Strength: clause.LockingStrengthUpdate}).First(&follow).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// follow does not exist
			return nil
		}
		return fmt.Errorf("failed to find follow: %w", err)
	}

	// check if follow is pending
	if !follow.Pending {
		return nil
	}

	// update follow to accepted
	follow.Pending = false
	if err := s.DB(ctx).Save(&follow).Error; err != nil {
		return fmt.Errorf("failed to update follow: %w", err)
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
		followID := s.urlForLocalFollowAccept(follow.ID)
		follow, err := s.createRemoteFollowObject(ctx, follow.URL.String, acceptorID, accepteeID)
		if err != nil {
			return fmt.Errorf("failed to create follow activity: %w", err)
		}
		acceptActivity := apub.NewAcceptActivityWithID(followID, follow.Followee.Apub.URL, follow.Activity.(apub.FollowActivity).AsAcceptable())
		if err := s.delivery.QueueActivity(ctx, acceptActivity, follow.Followee, []string{
			follow.Follower.Apub.PreferredInbox(),
		}); err != nil {
			return fmt.Errorf("failed to queue undo activity: %w", err)
		}
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

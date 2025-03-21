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
	ErrProfileNotFound = NewServiceError(404, "profile not found")
)

type ProfileUpdateParams struct {
	Nickname         *string
	Bio              *string
	AutoAcceptFollow *bool
	HideFollows      *bool
	AvatarUploadID   *types.UploadID // nil = no change, 0 = remove, otherwise = change
}

func (s *State) GetUserProfile(
	ctx context.Context,
	viewerID *types.UserID,
	userID types.UserID,
) (types.DetailedUser, error) {
	userRaw, err := s.FindUserByIDRaw(ctx, userID)
	if err != nil {
		return types.DetailedUser{}, err
	}
	if userRaw == nil {
		return types.DetailedUser{}, ErrProfileNotFound
	}

	followCount, err := s.getFollowerCount(ctx, userID)
	if err != nil {
		return types.DetailedUser{}, err
	}

	noteCount, err := s.getNoteCount(ctx, userID)
	if err != nil {
		return types.DetailedUser{}, err
	}

	var (
		isFollowing types.FollowState
		isFollowed  types.FollowState
		isBlocking  bool
		isBlocked   bool
	)
	if viewerID != nil && *viewerID != userID {
		isFollowing, err = s.GetFollowState(ctx, *viewerID, userID)
		if err != nil {
			return types.DetailedUser{}, err
		}
		isFollowed, err = s.GetFollowState(ctx, userID, *viewerID)
		if err != nil {
			return types.DetailedUser{}, err
		}
		isBlocking, err = s.IsBlocking(ctx, *viewerID, userID)
		if err != nil {
			return types.DetailedUser{}, err
		}
		isBlocked, err = s.IsBlocking(ctx, userID, *viewerID)
		if err != nil {
			return types.DetailedUser{}, err
		}
	}

	cleanBio := bioSanitizer.Sanitize(userRaw.Bio)

	return types.DetailedUser{
		Basic: types.SimpleUser{
			ID:       userRaw.ID,
			Username: userRaw.Username,
			Nickname: userRaw.Nickname,
			Domain:   userRaw.Domain,
			Bio:      cleanBio,
			Avatar:   userRaw.Avatar,
		},
		Details: types.DetailedUserModel{
			FollowCount:      followCount.FollowCount,
			FollowerCount:    followCount.FollowerCount,
			NoteCount:        noteCount,
			AutoFollowAccept: userRaw.AutoFollowAccept,
			HideFollows:      userRaw.HideFollows,
			RemoteURL:        sqlToString(userRaw.URL),
			RemoteViewURL:    sqlToString(userRaw.ViewURL),

			IsFollowing: isFollowing,
			IsFollowed:  isFollowed,
			IsBlocking:  isBlocking,
			IsBlocked:   isBlocked,
			IsMe:        viewerID != nil && *viewerID == userRaw.ID,
		},
	}, nil
}

type followCount struct {
	FollowCount   uint64
	FollowerCount uint64
}

func (s *State) getFollowerCount(ctx context.Context, userID types.UserID) (followCount, error) {
	var (
		follows   int64
		followers int64
	)
	if err := s.DB(ctx).Where("follower_id = ?", userID).Model(&db.ActualUserFollow{}).Count(&follows).Error; err != nil {
		return followCount{}, err
	}
	if err := s.DB(ctx).Where("followed_id = ?", userID).Model(&db.ActualUserFollow{}).Count(&followers).Error; err != nil {
		return followCount{}, err
	}
	return followCount{
		FollowCount:   uint64(follows),
		FollowerCount: uint64(followers),
	}, nil
}

func (s *State) getNoteCount(ctx context.Context, userID types.UserID) (uint64, error) {
	var count int64
	if err := s.DB(ctx).Where("author_id = ? AND deleted_at IS NULL", userID).Model(&db.Note{}).Count(&count).Error; err != nil {
		return 0, err
	}
	return uint64(count), nil
}

func (s *State) UpdateUserProfile(
	ctx context.Context,
	userID types.UserID,
	update ProfileUpdateParams,
) error {
	err := s.WithTransaction(func(tx *State) error {
		var user db.User
		err := tx.DB(ctx).Where("id = ?", userID).Clauses(clause.Locking{Strength: "UPDATE"}).First(&user).Error
		if err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return ErrProfileNotFound
			}
			return err
		}

		if update.Nickname != nil {
			user.Nickname = *update.Nickname
		}
		if update.Bio != nil {
			user.Bio = *update.Bio
		}
		if update.AutoAcceptFollow != nil {
			user.AutoFollowAccept = *update.AutoAcceptFollow
		}
		if update.HideFollows != nil {
			user.HideFollows = *update.HideFollows
		}
		if update.AvatarUploadID != nil {
			if *update.AvatarUploadID == (types.UploadID{}) {
				user.Avatar = nil
			} else {
				user.Avatar = update.AvatarUploadID
			}
		}

		err = tx.DB(ctx).Save(&user).Error
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return err
	}

	// TODO: apub Update
	return nil
}

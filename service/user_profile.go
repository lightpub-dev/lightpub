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
	"bytes"
	"context"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"github.com/rrivera/identicon"
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

type UserAvatar struct {
	HasUpload bool

	// If HasUpload is true
	UploadID types.UploadID
	// If HasUpload is false
	Ideticon []byte
}

func (s *State) GetUserAvatarFromUser(
	user types.SimpleUser,
) (UserAvatar, error) {
	if user.Avatar != nil {
		return UserAvatar{
			HasUpload: true,
			UploadID:  *user.Avatar,
		}, nil
	}

	identiconSource := user.ID.String()
	ig, err := identicon.New("", 5, 3)
	if err != nil {
		return UserAvatar{}, err
	}
	ii, err := ig.Draw(identiconSource)
	if err != nil {
		return UserAvatar{}, err
	}

	var buf bytes.Buffer
	if err := ii.Jpeg(300, 90, &buf); err != nil {
		return UserAvatar{}, err
	}

	return UserAvatar{
		HasUpload: false,
		Ideticon:  buf.Bytes(),
	}, nil
}

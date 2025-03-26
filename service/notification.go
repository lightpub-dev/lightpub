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
	"log/slog"
	"time"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/service/notification"
	"github.com/lightpub-dev/lightpub/types"
)

var (
	ErrExpiredNotification = NewServiceError(500, "expired notification")
)

func (s *State) GetUnreadNotificationCount(ctx context.Context, userID types.UserID) (uint64, error) {
	var count models.UnreadNotificationCount
	if err := s.DB(ctx).Where("user_id = ?", userID).First(&count).Error; err != nil {
		return 0, err
	}
	return count.UnreadCount, nil
}

func (s *State) AddNotification(ctx context.Context, userID types.UserID, body notification.Body) error {
	bodyJson, err := body.AsDB()
	if err != nil {
		return err
	}

	if err := s.DB(ctx).Create(&models.Notification{
		UserID: userID,
		Body:   bodyJson,
	}).Error; err != nil {
		return err
	}

	return nil
}

func (s *State) GetNotifications(ctx context.Context, userID types.UserID, limit int, page int) ([]notification.Notification, bool, error) {
	var ns []models.Notification
	if err := s.DB(ctx).Where("user_id = ?", userID).Order("created_at DESC").Limit(int(limit)).Offset(int(limit * page)).Find(&ns).Error; err != nil {
		return nil, false, err
	}

	mayHaveNext := len(ns) == limit

	expiredNotificationIDs := make([]types.NotificationID, 0)
	nss := make([]notification.Notification, 0, len(ns))
	for _, n := range ns {
		body, err := notification.ParseBody(n.Body)
		if err != nil {
			slog.WarnContext(ctx, "failed to parse notification body (old format?)", "notificationID", n.ID, "err", err)
			continue
		}
		if err := s.fillRelatedNotificationInfo(ctx, body); err != nil {
			slog.DebugContext(ctx, "failed to fill related notification info", "notificationID", n.ID, "err", err)
			expiredNotificationIDs = append(expiredNotificationIDs, types.NotificationID(n.ID))
			continue
		}
		n := notification.Notification{
			ID:        types.NotificationID(n.ID),
			CreatedAt: n.CreatedAt,
			Body:      body,
			ReadAt:    sqlToTimePtr(n.ReadAt),
		}
		nss = append(nss, n)
	}

	for _, expiredNotificationID := range expiredNotificationIDs {
		if err := s.deleteNotificationByID(ctx, expiredNotificationID); err != nil {
			return nil, false, err
		}
	}

	return nss, mayHaveNext, nil
}

func (s *State) fillRelatedNotificationInfo(ctx context.Context, body notification.Body) error {
	switch b := body.(type) {
	case *notification.Followed:
		user, err := s.FindUserByID(ctx, b.FollowerUserID)
		if err != nil {
			return err
		}
		if user == nil {
			return ErrExpiredNotification
		}
		b.FollowerUser = user
		return nil
	case *notification.FollowRequested:
		user, err := s.FindUserByID(ctx, b.RequesterUserID)
		if err != nil {
			return err
		}
		if user == nil {
			return ErrExpiredNotification
		}
		b.RequesterUser = user
		return nil
	case *notification.FollowAccepted:
		user, err := s.FindUserByID(ctx, b.AcceptorUserID)
		if err != nil {
			return err
		}
		if user == nil {
			return ErrExpiredNotification
		}
		b.AcceptorUser = user
		return nil
	case *notification.Replied:
		replier, err := s.FindUserByID(ctx, b.ReplierUserID)
		if err != nil {
			return err
		}
		if replier == nil {
			return ErrExpiredNotification
		}
		b.ReplierUser = replier

		replyNote, err := s.FindNoteByID(ctx, b.ReplyNoteID)
		if err != nil {
			return err
		}
		if replyNote == nil {
			return ErrExpiredNotification
		}
		b.ReplyNote = &notification.NoteData{
			NoteID:  replyNote.ID,
			ViewURL: "", // TODO: fill this
		}

		repliedNote, err := s.FindNoteByID(ctx, b.RepliedNoteID)
		if err != nil {
			return err
		}
		if repliedNote == nil {
			return ErrExpiredNotification
		}
		b.RepliedNote = &notification.NoteData{
			NoteID:  repliedNote.ID,
			ViewURL: "", // TODO: fill this
		}
		return nil
	case *notification.Mentioned:
		mentioner, err := s.FindUserByID(ctx, b.MentionerUserID)
		if err != nil {
			return err
		}
		if mentioner == nil {
			return ErrExpiredNotification
		}
		b.MentionerUser = mentioner

		mentionNote, err := s.FindNoteByID(ctx, b.MentionNoteID)
		if err != nil {
			return err
		}
		if mentionNote == nil {
			return ErrExpiredNotification
		}
		b.MentionNote = &notification.NoteData{
			NoteID:  mentionNote.ID,
			ViewURL: "", // TODO: fill this
		}
		return nil
	case *notification.Renote:
		renoter, err := s.FindUserByID(ctx, b.RenoterUserID)
		if err != nil {
			return err
		}
		if renoter == nil {
			return ErrExpiredNotification
		}

		b.RenoterUser = renoter
		return nil
	}

	return fmt.Errorf("unknown notification body type: %T", body)
}

func (s *State) deleteNotificationByID(ctx context.Context, notificationID types.NotificationID) error {
	if err := s.DB(ctx).Unscoped().Delete(&models.Notification{
		ID: int(notificationID),
	}).Error; err != nil {
		return err
	}
	return nil
}

func (s *State) ReadNotificationID(ctx context.Context, userID types.UserID, notificationID types.NotificationID) error {
	if err := s.DB(ctx).Model(&models.Notification{}).Where("id = ? AND user_id = ?", notificationID, userID).Update("read_at", time.Now()).Error; err != nil {
		return err
	}
	return nil
}

func (s *State) ReadAllNotifications(ctx context.Context, userID types.UserID) error {
	if err := s.DB(ctx).Model(&models.Notification{}).Where("user_id = ? AND read_at IS NULL", userID).Update("read_at", time.Now()).Error; err != nil {
		return err
	}
	return nil
}

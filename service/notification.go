package service

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/service/notification"
	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) GetUnreadNotificationCount(ctx context.Context, userID types.UserID) (uint64, error) {
	var count db.UnreadNotificationCount
	if err := s.DB(ctx).Where("user_id = ?", userID).First(&count).Error; err != nil {
		return 0, err
	}
	return count.UnreadCount, nil
}

func (s *State) AddNotification(ctx context.Context, userID types.UserID, body notification.Body) error {
	bodyJson, err := notification.Stringify(body)
	if err != nil {
		return err
	}

	if err := s.DB(ctx).Create(&db.Notification{
		UserID: userID,
		Body:   bodyJson,
	}).Error; err != nil {
		return err
	}

	return nil
}

func (s *State) ReadNotificationID(ctx context.Context, notificationID types.NotificationID) error {
	if err := s.DB(ctx).Model(&db.Notification{}).Where("id = ?", notificationID).Update("read_at", time.Now()).Error; err != nil {
		return err
	}
	return nil
}

func (s *State) ReadAllNotifications(ctx context.Context, userID types.UserID) error {
	if err := s.DB(ctx).Model(&db.Notification{}).Where("user_id = ? AND read_at IS NULL", userID).Update("read_at", time.Now()).Error; err != nil {
		return err
	}
	return nil
}

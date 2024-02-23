package users

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

func UpdateProfile(ctx context.Context, conn db.DBConn, userID db.UUID, req *models.UserProfileUpdate) error {
	dbconn := conn.DB()
	tx := dbconn.Begin()
	defer tx.Rollback()

	if req.Nickname != nil {
		err := tx.Model(&db.User{}).Where("id = ?", userID).Update("nickname", *req.Nickname).Error
		if err != nil {
			return err
		}
	}

	if req.Bio != nil {
		err := tx.Model(&db.User{}).Where("id = ?", userID).Update("bio", *req.Bio).Error
		if err != nil {
			return err
		}
	}

	if req.Labels != nil {
		// delete all existing labels
		tx.Delete(&db.UserLabelDB{}, "user_id = ?", userID)

		for i, label := range req.Labels {
			data := db.UserLabelDB{
				UserID: userID,
				Order:  i,
				Key:    label.Key,
				Value:  label.Value,
			}
			err := tx.Create(&data).Error
			if err != nil {
				return err
			}
		}
	}

	return tx.Commit().Error
}

func GetProfile(ctx context.Context, conn db.DBConn, userSpec string, viewerID db.UUID) (*db.FullUser, error) {
	basicUser, err := FindIDByUsername(ctx, conn, userSpec)
	if err != nil {
		return nil, err
	}

	if basicUser == nil {
		return nil, nil
	}

	var profile db.FullUser
	profile.User = *basicUser

	// fetch labels
	var labels []db.UserLabelDB
	err = conn.DB().Find(&labels, "user_id = ?", basicUser.ID).Order("order ASC").Error
	if err != nil {
		return nil, err
	}
	profile.Labels = labels

	// fetch is_following
	if viewerID != (db.UUID{}) {
		var isFollowingCount int64
		err = conn.DB().Model(&db.UserFollow{}).Where("follower_id = ? AND followee_id = ?", viewerID, basicUser.ID).Count(&isFollowingCount).Error
		if err != nil {
			return nil, err
		}
		profile.IsFollowingByViewer = isFollowingCount > 0
	}

	// follower count
	err = conn.DB().Model(&db.UserFollow{}).Where("followee_id = ?", basicUser.ID).Count(&profile.Followers).Error
	if err != nil {
		return nil, err
	}

	err = conn.DB().Model(&db.UserFollow{}).Where("follower_id = ?", basicUser.ID).Count(&profile.Following).Error
	if err != nil {
		return nil, err
	}

	// post count
	err = conn.DB().Model(&db.Post{}).Where("poster_id = ?", basicUser.ID).Count(&profile.PostCount).Error
	if err != nil {
		return nil, err
	}

	return &profile, nil
}

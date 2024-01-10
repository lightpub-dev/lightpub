package users

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

func UpdateProfile(ctx context.Context, conn db.DBConn, userID string, req *models.UserProfileUpdate) error {
	db := conn.DB()
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}
	defer tx.Rollback()

	if req.Bio != nil {
		_, err = tx.ExecContext(ctx, "UPDATE UserProfile SET bio=? WHERE user_id=UUID_TO_BIN(?)", *req.Bio, userID)
		if err != nil {
			return err
		}
	}

	if req.Labels != nil {
		// delete all existing labels
		_, err = tx.ExecContext(ctx, "DELETE FROM UserLabel WHERE user_id=UUID_TO_BIN(?)", userID)
		if err != nil {
			return err
		}

		for i, label := range req.Labels {
			_, err = tx.ExecContext(ctx, "INSERT INTO UserLabel(user_id, `order`, `key`, `value`) VALUES (UUID_TO_BIN(?), ?, ?, ?)", userID, i, label.Key, label.Value)
			if err != nil {
				return err
			}
		}
	}

	return tx.Commit()
}

func GetProfile(ctx context.Context, conn db.DBConn, userSpec string, viewerID string) (*models.FullUser, error) {
	basicUser, err := FindIDByUsername(ctx, conn, userSpec)
	if err != nil {
		return nil, err
	}

	if basicUser == nil {
		return nil, nil
	}

	var profile models.FullUser
	profile.User = *basicUser

	// fetch Bio
	err = conn.DB().GetContext(ctx, &profile.Bio, "SELECT bio FROM UserProfile WHERE user_id=UUID_TO_BIN(?)", basicUser.ID)
	if err != nil {
		return nil, err
	}

	// fetch labels
	var labels []models.UserLabelDB
	err = conn.DB().SelectContext(ctx, &labels, "SELECT `order`,`key`,`value` FROM UserLabel WHERE user_id=UUID_TO_BIN(?) ORDER BY `order` ASC", basicUser.ID)
	if err != nil {
		return nil, err
	}
	profile.Labels = labels

	// fetch is_following
	if viewerID != "" {
		err = conn.DB().GetContext(ctx, &profile.IsFollowingByViewer, "SELECT COUNT(*)>0 FROM UserFollow WHERE follower_id=UUID_TO_BIN(?) AND followee_id=UUID_TO_BIN(?) LIMIT 1", viewerID, basicUser.ID)
		if err != nil {
			return nil, err
		}
	}

	return &profile, nil
}

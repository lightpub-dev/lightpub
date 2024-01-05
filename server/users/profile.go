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
			_, err = tx.ExecContext(ctx, "INSERT INTO UserLabel(user_id, order, key, value) VALUES (UUID_TO_BIN(?), ?, ?, ?)", userID, i, label.Key, label.Value)
			if err != nil {
				return err
			}
		}
	}

	return tx.Commit()
}

package reactions

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
)

func FindReactionByID(context context.Context, conn db.DBConn, name string) (*db.Reaction, error) {
	var reaction *db.Reaction
	err := conn.DB().WithContext(context).Find(&reaction, "name = ?", name).Error
	if err != nil {
		return nil, err
	}
	return reaction, nil
}

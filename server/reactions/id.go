package reactions

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
)

type FindReactionService interface {
	FindReactionByID(context.Context, string) (*db.Reaction, error)
}

type DBFindReactionService struct {
	conn db.DBConn
}

func ProvideDBFindReactionService(conn db.DBConn) *DBFindReactionService {
	return &DBFindReactionService{conn}
}

func (s *DBFindReactionService) FindReactionByID(name string) (*db.Reaction, error) {
	conn := s.conn.DB
	context := s.conn.Ctx.Ctx

	var reaction *db.Reaction
	err := conn.WithContext(context).Find(&reaction, "name = ?", name).Error
	if err != nil {
		return nil, err
	}
	return reaction, nil
}

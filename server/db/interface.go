package db

import (
	"github.com/jmoiron/sqlx"
)

type DBConn interface {
	DB() *sqlx.DB
}

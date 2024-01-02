package db

import (
	"context"
	"database/sql"

	"github.com/jmoiron/sqlx"
)

type DBOrTx interface {
	GetContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error
	SelectContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error
	ExecContext(ctx context.Context, query string, args ...interface{}) (sql.Result, error)
}

type DBWrapper struct {
	*sqlx.DB
}

type TxWrapper struct {
	*sqlx.Tx
}

package db

import (
	"context"
	"database/sql"
	"errors"

	"github.com/jmoiron/sqlx"
)

type DBOrTx interface {
	GetContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error
	SelectContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error
	ExecContext(ctx context.Context, query string, args ...interface{}) (sql.Result, error)
	isTx() bool
}

type DBWrapper struct {
	*sqlx.DB
}

func (db *DBWrapper) isTx() bool {
	return false
}

type TxWrapper struct {
	*sqlx.Tx
}

func (tx *TxWrapper) isTx() bool {
	return true
}

type DBIO struct {
	DBOrTx
	Ctx context.Context
}

func (dbio *DBIO) GetContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error {
	return dbio.DBOrTx.GetContext(ctx, dest, query, args...)
}

func (dbio *DBIO) SelectContext(ctx context.Context, dest interface{}, query string, args ...interface{}) error {
	return dbio.DBOrTx.SelectContext(ctx, dest, query, args...)
}

func (dbio *DBIO) ExecContext(ctx context.Context, query string, args ...interface{}) (sql.Result, error) {
	return dbio.DBOrTx.ExecContext(ctx, query, args...)
}

func (dbio *DBIO) NamedExecContext(ctx context.Context, query string, arg interface{}) (sql.Result, error) {
	return dbio.DBOrTx.(*TxWrapper).NamedExecContext(ctx, query, arg)
}

func (dbio *DBIO) GetDefaultContext(dest interface{}, query string, args ...interface{}) error {
	return dbio.GetContext(dbio.Ctx, dest, query, args...)
}

func (dbio *DBIO) SelectDefaultContext(dest interface{}, query string, args ...interface{}) error {
	return dbio.SelectContext(dbio.Ctx, dest, query, args...)
}

func (dbio *DBIO) ExecDefaultContext(query string, args ...interface{}) (sql.Result, error) {
	return dbio.ExecContext(dbio.Ctx, query, args...)
}

func (dbio *DBIO) NamedExecDefaultContext(query string, arg interface{}) (sql.Result, error) {
	return dbio.NamedExecContext(dbio.Ctx, query, arg)
}

func (dbio *DBIO) Beginx() (*DBIO, error) {
	isTx := dbio.DBOrTx.isTx()
	if isTx {
		return dbio, nil
	}

	tx, err := dbio.DBOrTx.(*DBWrapper).Beginx()
	if err != nil {
		return nil, err
	}
	return &DBIO{DBOrTx: &TxWrapper{tx}, Ctx: dbio.Ctx}, nil
}

func (dbio *DBIO) Rollback() error {
	isTx := dbio.DBOrTx.isTx()
	if !isTx {
		return errors.New("not a transaction")
	}

	tx := dbio.DBOrTx.(*TxWrapper)
	err := tx.Rollback()
	if err != nil {
		return err
	}
	return nil
}

func (dbio *DBIO) Commit() error {
	isTx := dbio.DBOrTx.isTx()
	if !isTx {
		return errors.New("not a transaction")
	}

	tx := dbio.DBOrTx.(*TxWrapper)
	err := tx.Commit()
	if err != nil {
		return err
	}
	return nil
}

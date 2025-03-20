package service

import (
	"context"

	"gorm.io/gorm"
)

var (
	ErrAlreadyInTx = NewInternalServerError("already in transaction")
	ErrNotInTx     = NewInternalServerError("not in transaction")
)

type State struct {
	db   *gorm.DB
	inTx bool

	devMode bool
}

func (s *State) WithTransaction(f func(tx *State) error) error {
	if s.inTx {
		return ErrAlreadyInTx
	}

	copied := *s
	copied.db = s.db.Begin()
	copied.inTx = true

	var err error
	defer func() {
		if r := recover(); r != nil {
			if copied.inTx {
				copied.db.Rollback()
			}
			panic(r)
		}
		if copied.inTx {
			if err != nil {
				copied.db.Rollback()
			} else {
				copied.db.Commit()
			}
		}
	}()

	err = f(&copied)
	return err
}

func (s *State) DB(ctx context.Context) *gorm.DB {
	return s.db.WithContext(ctx)
}

func (s *State) DevMode() bool {
	return s.devMode
}

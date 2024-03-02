package db

import (
	"gorm.io/gorm"
)

type DBConn struct {
	Ctx Context
	DB  *gorm.DB
}

func ProvideDBConn(ctx Context, db *gorm.DB) DBConn {
	return DBConn{ctx, db}
}

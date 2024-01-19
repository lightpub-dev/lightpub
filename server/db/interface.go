package db

import (
	"gorm.io/gorm"
)

type DBConn interface {
	DB() *gorm.DB
}

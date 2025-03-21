package service

import (
	"context"
	"fmt"

	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	"resty.dev/v3"
)

var (
	ErrAlreadyInTx = NewInternalServerError("already in transaction")
	ErrNotInTx     = NewInternalServerError("not in transaction")
)

type State struct {
	db   *gorm.DB
	inTx bool

	uploadFetchClient *resty.Client
	// remoteUploadCache kv.Cache

	uploadDir string
	devMode   bool
}

type Config struct {
	Database  DatabaseConfig `yaml:"database"`
	UploadDir string         `yaml:"upload_dir"`
	DevMode   bool           `yaml:"dev_mode"`
}

type DatabaseConfig struct {
	Host string `yaml:"host"`
	Port int    `yaml:"port"`
	User string `yaml:"user"`
	Pass string `yaml:"pass"`
	Name string `yaml:"name"`
}

func NewStateFromConfig(config Config) *State {
	db := dbConnect(config.Database)
	return &State{
		db:                db,
		uploadFetchClient: resty.New(),
		uploadDir:         config.UploadDir,
		devMode:           config.DevMode,
	}
}

func dbConnect(config DatabaseConfig) *gorm.DB {
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%d)/%s?charset=utf8mb4&parseTime=true&loc=UTC", config.User, config.Pass, config.Host, config.Port, config.Name)
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{})
	if err != nil {
		panic(err)
	}
	return db
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

func (s *State) getUploadsDir() string {
	return s.uploadDir
}

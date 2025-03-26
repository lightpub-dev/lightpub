/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

package service

import (
	"context"
	"fmt"
	"net/url"

	"github.com/lightpub-dev/lightpub/apub"
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

	delivery *apub.DeliveryState

	uploadFetchClient *resty.Client
	// remoteUploadCache kv.Cache

	baseURL   string
	uploadDir string
	devMode   bool
}

type Config struct {
	Database  DatabaseConfig `yaml:"database"`
	BaseURL   string         `yaml:"base_url"`
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
		baseURL:           config.BaseURL,
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

func (s *State) BaseURL() *url.URL {
	// TODO: cache result
	u, err := url.Parse(s.baseURL)
	if err != nil {
		panic(err)
	}
	return u
}

func (s *State) MyDomain() string {
	// TODO: cache result
	url, err := url.Parse(s.baseURL)
	if err != nil {
		panic(err)
	}
	return url.Host
}

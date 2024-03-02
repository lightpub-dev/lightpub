package testutils

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"

	_ "github.com/go-sql-driver/mysql"
	"github.com/jmoiron/sqlx"
	"github.com/labstack/echo/v4"
	"github.com/labstack/gommon/log"
	"github.com/lightpub-dev/lightpub/api"
	"github.com/lightpub-dev/lightpub/db"
)

var (
	TableNames = []string{
		"User",
		"UserFollow",
		"UserToken",
		"Post",
		"PostHashtag",
		"PostFavorite",
		"PostMention",
		"PostReaction",
		"PostAttachment",
		"PostPoll",
		"PollChoice",
		"PollVote",
	}
)

func TruncateAll(conn db.DBConnectionInfo) error {
	db, err := sqlx.Open("mysql", fmt.Sprintf("%s:%s@tcp(%s:%s)/%s?parseTime=true", conn.Username, conn.Password, conn.Host, conn.Port, conn.Database))
	if err != nil {
		return err
	}

	if err := db.Ping(); err != nil {
		return err
	}

	if _, err := db.Exec("SET FOREIGN_KEY_CHECKS=0"); err != nil {
		return err
	}

	for _, tableName := range TableNames {
		if _, err := db.Exec(fmt.Sprintf("TRUNCATE TABLE %s", tableName)); err != nil {
			return err
		}
	}

	if _, err := db.Exec("SET FOREIGN_KEY_CHECKS=1"); err != nil {
		return err
	}

	return nil
}

func DefaultDBConnection() db.DBConnectionInfo {
	return db.DBConnectionInfo{
		Host:      "localhost",
		Port:      "3307",
		Username:  "lightpub",
		Password:  "lightpub",
		Database:  "lightpub",
		RedisHost: "localhost",
		RedisPort: "6379",
	}
}

func DefaultEcho() (*echo.Echo, *api.Handler) {
	conn := DefaultDBConnection()
	db, err := db.ConnectDB(conn)
	if err != nil {
		panic(err)
	}

	h := api.NewHandler(db.DB, db.RDB, "http://example.com")
	return api.BuildEcho(h, api.EchoOptions{
		LogLevel: log.WARN,
	}), h
}

func NewJSONBody(s interface{}) io.Reader {
	b, err := json.Marshal(s)
	if err != nil {
		panic(err)
	}
	return bytes.NewReader(b)
}

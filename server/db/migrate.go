package db

import (
	"fmt"
	"log"

	"github.com/golang-migrate/migrate/v4"
	_ "github.com/golang-migrate/migrate/v4/database/mysql"
	_ "github.com/golang-migrate/migrate/v4/source/file"
)

type migrateLogger struct{}

func (m migrateLogger) Printf(format string, v ...interface{}) {
	log.Printf(format, v...)
}

func (m migrateLogger) Verbose() bool {
	return true
}

func MigrateToLatest(conn DBConnectionInfo, migrationsDir string, logging bool) error {
	m, err := migrate.New(
		fmt.Sprintf("file://%s", migrationsDir),
		fmt.Sprintf("mysql://%s:%s@tcp(%s:%s)/%s?parseTime=true",

			conn.Username,
			conn.Password,
			conn.Host,
			conn.Port,
			conn.Database,
		),
	)
	if err != nil {
		return err
	}
	if logging {
		m.Log = migrateLogger{}
	} else {
		m.Log = nil
	}

	if err != nil {
		return err
	}

	if err := m.Up(); err != nil {
		if err == migrate.ErrNoChange {
			return nil
		}
		return err
	}

	return nil
}

package main

import (
	"os"

	_ "github.com/go-sql-driver/mysql"
	"github.com/labstack/gommon/log"
	api "github.com/lightpub-dev/lightpub/api"
	d "github.com/lightpub-dev/lightpub/db"
)

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}

func main2() {
	// get config from env
	dbHost := getEnv("DB_HOST", "localhost")
	dbPort := (getEnv("DB_PORT", "3306"))
	dbUsername := getEnv("DB_USERNAME", "lightpub")
	dbPassword := getEnv("DB_PASSWORD", "lightpub")
	dbName := getEnv("DB_NAME", "lightpub")
	redisHost := getEnv("REDIS_HOST", "localhost")
	redisPort := getEnv("REDIS_PORT", "6379")
	conn := d.DBConnectionInfo{
		Host:      dbHost,
		Port:      dbPort,
		Username:  dbUsername,
		Password:  dbPassword,
		Database:  dbName,
		RedisHost: redisHost,
		RedisPort: redisPort,
	}
	db, err := d.ConnectDB(conn)
	if err != nil {
		panic(err)
	}

	// migrate
	if err := d.MigrateToLatest(conn, "./migrations", true); err != nil {
		panic(err)
	}

	e := api.BuildEcho(api.NewHandler(db.DB, db.RDB), api.EchoOptions{LogLevel: log.INFO})

	e.Logger.Fatal(e.Start(":1323"))
}

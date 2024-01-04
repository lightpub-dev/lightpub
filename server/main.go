package main

import (
	"os"

	_ "github.com/go-sql-driver/mysql"
	api "github.com/lightpub-dev/lightpub/api"
	"github.com/lightpub-dev/lightpub/db"
)

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}

func main() {
	// get config from env
	dbHost := getEnv("DB_HOST", "localhost")
	dbPort := (getEnv("DB_PORT", "3306"))
	dbUsername := getEnv("DB_USERNAME", "lightpub")
	dbPassword := getEnv("DB_PASSWORD", "lightpub")
	dbName := getEnv("DB_NAME", "lightpub")
	redisHost := getEnv("REDIS_HOST", "localhost")
	redisPort := getEnv("REDIS_PORT", "6379")
	conn := db.DBConnectionInfo{
		Host:      dbHost,
		Port:      dbPort,
		Username:  dbUsername,
		Password:  dbPassword,
		Database:  dbName,
		RedisHost: redisHost,
		RedisPort: redisPort,
	}
	db, err := db.ConnectDB(conn)
	if err != nil {
		panic(err)
	}

	e := api.BuildEcho(api.NewHandler(db.DB, db.RDB))

	e.Logger.Fatal(e.Start(":1323"))
}

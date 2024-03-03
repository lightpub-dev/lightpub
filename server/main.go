package main

import (
	"flag"
	"os"

	_ "github.com/go-sql-driver/mysql"
	"github.com/labstack/gommon/log"
	api "github.com/lightpub-dev/lightpub/api"
	d "github.com/lightpub-dev/lightpub/db"
)

var doMigrate = flag.Bool("migrate", false, "run migrations and not start the server")

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}

func main() {
	flag.Parse()

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
	if *doMigrate {
		if err := db.MigrateToLatest(); err != nil {
			panic(err)
		}
		log.Print("migration done")
		return
	}

	e := api.BuildEcho(api.NewHandler(db.DB, db.RDB, "https://lightpub.tinax.local"), api.EchoOptions{LogLevel: log.DEBUG})

	e.Logger.Fatal(e.Start(":8000"))
}

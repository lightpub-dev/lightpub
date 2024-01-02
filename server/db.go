package main

import (
	"context"

	"github.com/jmoiron/sqlx"
	"github.com/redis/go-redis/v9"
)

var (
	db  *sqlx.DB
	rdb *redis.Client
)

func mustConnectDB() {
	// connect to mysql
	var err error
	db, err = sqlx.Connect("mysql", "lightpub:lightpub@tcp(localhost:3306)/lightpub?parseTime=true")
	if err != nil {
		panic(err)
	}
}

func mustConnectRedis() {
	rdb = redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "", // no password set
		DB:       0,  // use default DB
	})

	// ping
	_, err := rdb.Ping(context.Background()).Result()
	if err != nil {
		panic(err)
	}

	// flush all
	// TODO: remove this in production
	_, err = rdb.FlushAll(context.Background()).Result()
	if err != nil {
		panic(err)
	}
}

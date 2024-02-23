package db

import (
	"context"
	"fmt"

	"github.com/redis/go-redis/v9"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

type DBConnectionInfo struct {
	Host      string
	Port      string
	Username  string
	Password  string
	Database  string
	RedisHost string
	RedisPort string
}

type DBConnectResult struct {
	DB  *gorm.DB
	RDB *redis.Client
}

func ConnectDB(connectDB DBConnectionInfo) (*DBConnectResult, error) {
	var err error
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%s)/%s?parseTime=true", connectDB.Username, connectDB.Password, connectDB.Host, connectDB.Port, connectDB.Database)
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Info),
	})
	if err != nil {
		return nil, err
	}

	rdb := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%s", connectDB.RedisHost, connectDB.RedisPort),
		Password: "", // no password set
		DB:       0,  // use default DB
	})

	// ping
	_, err = rdb.Ping(context.Background()).Result()
	if err != nil {
		return nil, err
	}

	// flush all
	// TODO: remove this in production
	_, err = rdb.FlushAll(context.Background()).Result()
	if err != nil {
		return nil, err
	}

	return &DBConnectResult{
		DB:  db,
		RDB: rdb,
	}, nil
}

func (d *DBConnectResult) MigrateToLatest() error {
	return d.DB.AutoMigrate(
		&User{},
		&UserLabelDB{},
		&UserToken{},
		&Post{},
		&PostAttachment{},
		&PostFavorite{},
		&PostHashtag{},
		&PostPoll{},
		&PostReaction{},
		&PollChoice{},
		&PollVote{},
		&PostMention{},
		&UserFollow{},
	)
}

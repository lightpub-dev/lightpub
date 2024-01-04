package main

import (
	"os"

	"github.com/go-playground/validator/v10"
	_ "github.com/go-sql-driver/mysql"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	httpSwagger "github.com/swaggo/http-swagger/v2"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

func buildEcho(h *Handler) *echo.Echo {
	e := echo.New()

	// setup logger
	e.Use(middleware.Logger())

	// CORS
	e.Use(middleware.CORS())

	// No Auth APIs
	e.POST("/login", h.postLogin)
	e.POST("/register", h.postRegister)

	authed := e.Group("", h.authMiddleware(false))
	unAuthed := e.Group("", h.authMiddleware(true))
	// APIs with auth
	// Posts
	authed.POST("/post", h.postPost)
	unAuthed.GET("/post/:post_id", h.getPost)
	authed.POST("/post/:post_id/reply", h.postReply)
	authed.POST("/post/:post_id/repost", h.postRepost)
	authed.PUT("/post/:post_id/quote", h.postQuote)
	authed.PUT("/post/:post_id/reaction/:reaction", h.putPostReaction)
	authed.DELETE("/post/:post_id/reaction/:reaction", h.deletePostReaction)
	authed.PUT("/post/:post_id/favorite", h.putPostFavorite)
	authed.DELETE("/post/:post_id/favorite", h.deletePostFavorite)
	authed.PUT("/post/:post_id/bookmark", h.putPostBookmark)
	authed.DELETE("/post/:post_id/bookmark", h.deletePostBookmark)

	// Users
	unAuthed.GET("/user/:username/posts", h.getUserPosts)
	unAuthed.GET("/user/:username/followers", h.getUserFollowers)
	unAuthed.GET("/user/:username/following", h.getUserFollowing)
	authed.PUT("/user/:username/follow", h.followAUser)
	authed.DELETE("/user/:username/follow", h.unfollowAUser)

	// Timeline
	authed.GET("/timeline", h.getTimeline)

	// webfinger
	unAuthed.GET("/.well-known/webfinger", h.getWebfinger)

	// swagger
	e.GET("/docs/*", echo.WrapHandler(httpSwagger.Handler(
		httpSwagger.URL("http://localhost:1323/openapi.yml"), //The url pointing to API definition"
	)))
	e.GET("/openapi.yml", func(c echo.Context) error {
		return c.File("./openapi.yml")
	})

	return e
}

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
	conn := dbConnectionInfo{
		Host:      dbHost,
		Port:      dbPort,
		Username:  dbUsername,
		Password:  dbPassword,
		Database:  dbName,
		RedisHost: redisHost,
		RedisPort: redisPort,
	}
	db, err := connectDB(conn)
	if err != nil {
		panic(err)
	}

	e := buildEcho(db)

	e.Logger.Fatal(e.Start(":1323"))
}

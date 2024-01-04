package api

import (
	"github.com/go-playground/validator/v10"
	"github.com/jmoiron/sqlx"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/labstack/gommon/log"
	"github.com/redis/go-redis/v9"
	httpSwagger "github.com/swaggo/http-swagger/v2"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

type Handler struct {
	DB  *sqlx.DB
	RDB *redis.Client
}

func NewHandler(db *sqlx.DB, rdb *redis.Client) *Handler {
	return &Handler{
		DB:  db,
		RDB: rdb,
	}
}

type dbconn struct {
	D *sqlx.DB
}

func (h *Handler) MakeDB() *dbconn {
	return &dbconn{
		D: h.DB,
	}
}

func (d *dbconn) DB() *sqlx.DB {
	return d.D
}

type EchoOptions struct {
	LogLevel log.Lvl
}

func BuildEcho(h *Handler, options EchoOptions) *echo.Echo {
	e := echo.New()

	// setup logger
	e.Use(middleware.Logger())
	e.Logger.SetLevel(options.LogLevel)

	// CORS
	e.Use(middleware.CORS())

	// No Auth APIs
	e.POST("/login", h.PostLogin)
	e.POST("/register", h.PostRegister)

	authed := e.Group("", h.AuthMiddleware(false))
	unAuthed := e.Group("", h.AuthMiddleware(true))
	// APIs with auth
	// Posts
	authed.POST("/post", h.PostPost)
	unAuthed.GET("/post/:post_id", h.GetPost)
	authed.POST("/post/:post_id/reply", h.PostReply)
	authed.POST("/post/:post_id/repost", h.PostRepost)
	authed.PUT("/post/:post_id/quote", h.PostQuote)
	authed.PUT("/post/:post_id/reaction/:reaction", h.PutPostReaction)
	authed.DELETE("/post/:post_id/reaction/:reaction", h.DeletePostReaction)
	authed.PUT("/post/:post_id/favorite", h.PutPostFavorite)
	authed.DELETE("/post/:post_id/favorite", h.DeletePostFavorite)
	authed.PUT("/post/:post_id/bookmark", h.PutPostBookmark)
	authed.DELETE("/post/:post_id/bookmark", h.DeletePostBookmark)

	// Users
	unAuthed.GET("/user/:username/posts", h.GetUserPosts)
	unAuthed.GET("/user/:username/followers", h.GetUserFollowers)
	unAuthed.GET("/user/:username/following", h.GetUserFollowing)
	authed.PUT("/user/:username/follow", h.FollowAUser)
	authed.DELETE("/user/:username/follow", h.UnfollowAUser)

	// Timeline
	authed.GET("/timeline", h.GetTimeline)

	// webfinger
	unAuthed.GET("/.well-known/webfinger", h.GetWebfinger)

	// swagger
	e.GET("/docs/*", echo.WrapHandler(httpSwagger.Handler(
		httpSwagger.URL("http://localhost:1323/openapi.yml"), //The url pointing to API definition"
	)))
	e.GET("/openapi.yml", func(c echo.Context) error {
		return c.File("./openapi.yml")
	})

	return e
}

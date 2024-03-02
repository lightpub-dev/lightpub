package api

import (
	"html/template"
	"io"

	"github.com/go-playground/validator/v10"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/labstack/gommon/log"
	"github.com/redis/go-redis/v9"
	httpSwagger "github.com/swaggo/http-swagger/v2"
	"gorm.io/gorm"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

type Handler struct {
	DB      *gorm.DB
	RDB     *redis.Client
	BaseURL string
}

func NewHandler(db *gorm.DB, rdb *redis.Client, baseURL string) *Handler {
	return &Handler{
		DB:      db,
		RDB:     rdb,
		BaseURL: baseURL,
	}
}

type dbconn struct {
	D *gorm.DB
}

func (h *Handler) MakeDB() *dbconn {
	return &dbconn{
		D: h.DB,
	}
}

func (d *dbconn) DB() *gorm.DB {
	return d.D
}

type EchoOptions struct {
	LogLevel log.Lvl
}

func BuildEcho(h *Handler, options EchoOptions) *echo.Echo {
	e := echo.New()

	// setup templates
	t := &Template{
		templates: template.Must(template.ParseGlob("templates/*")),
	}
	e.Renderer = t

	// setup logger
	e.Use(middleware.Logger())
	e.Logger.SetLevel(options.LogLevel)

	// CORS
	e.Use(middleware.CORS())

	// No Auth APIs
	e.POST("/login", h.PostLogin)
	e.POST("/register", h.PostRegister)

	// authed := e.Group("", h.AuthMiddleware(false))
	// unAuthed := e.Group("", h.AuthMiddleware(true))
	// APIs with auth
	// Posts
	// authed.POST("/post", h.PostPost)
	// unAuthed.GET("/post/:post_id", h.GetPost)
	// authed.PUT("/post/:post_id/reaction/:reaction", h.PutPostReaction)
	// authed.DELETE("/post/:post_id/reaction/:reaction", h.DeletePostReaction)
	// authed.PUT("/post/:post_id/favorite", h.PutPostFavorite)
	// authed.DELETE("/post/:post_id/favorite", h.DeletePostFavorite)
	// authed.PUT("/post/:post_id/bookmark", h.PutPostBookmark)
	// authed.DELETE("/post/:post_id/bookmark", h.DeletePostBookmark)

	// Users
	// unAuthed.GET("/user/:username/posts", h.GetUserPosts)
	// unAuthed.GET("/user/:username/followers", h.GetUserFollowers)
	// unAuthed.GET("/user/:username/following", h.GetUserFollowing)
	// authed.PUT("/user/:username/follow", h.FollowAUser)
	// authed.DELETE("/user/:username/follow", h.UnfollowAUser)
	// authed.PATCH("/user/:userspec", h.PutUser)
	// unAuthed.GET("/user/:username", h.GetUser)
	// unAuthed.POST("/user/:userspec/inbox", h.UserInbox)
	// unAuthed.GET("/user/:userspec/outbox", h.UserOutbox)

	// Timeline
	// authed.GET("/timeline", h.GetTimeline)

	// Trend
	// unAuthed.GET("/trend", h.GetTrend)
	// unAuthed.GET("/trend/posts", h.GetTrendPosts)

	// webfinger
	// unAuthed.GET("/.well-known/webfinger", h.GetWebfinger)
	// unAuthed.GET("/.well-known/nodeinfo", h.GetNodeInfo)
	// unAuthed.GET("/.well-known/host-meta", h.GetHostMeta)
	// unAuthed.GET("/nodeinfo/2.0", h.Nodeinfo20)
	// unAuthed.GET("/nodeinfo/2.1", h.Nodeinfo21)

	// swagger
	e.GET("/docs/*", echo.WrapHandler(httpSwagger.Handler(
		httpSwagger.URL("http://localhost:1323/openapi.yml"), //The url pointing to API definition"
	)))
	e.GET("/openapi.yml", func(c echo.Context) error {
		return c.File("./openapi.yml")
	})

	return e
}

type Template struct {
	templates *template.Template
}

func (t *Template) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.templates.ExecuteTemplate(w, name, data)
}

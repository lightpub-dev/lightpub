package main

import (
	"github.com/go-playground/validator/v10"
	_ "github.com/go-sql-driver/mysql"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

func main() {
	mustConnectDB()
	mustConnectRedis()

	e := echo.New()

	// setup logger
	e.Use(middleware.Logger())

	// No Auth APIs
	e.POST("/login", postLogin)
	e.POST("/register", postRegister)

	authed := e.Group("", authMiddleware(false))
	unAuthed := e.Group("", authMiddleware(true))
	// APIs with auth
	// Posts
	authed.POST("/post", postPost)
	unAuthed.GET("/post/:post_id", getPost)
	authed.POST("/post/:post_id/reply", postReply)
	authed.POST("/post/:post_id/repost", postRepost)
	authed.PUT("/post/:post_id/quote", postQuote)
	authed.PUT("/post/:post_id/reaction/:reaction", putPostReaction)
	authed.DELETE("/post/:post_id/reaction/:reaction", deletePostReaction)
	authed.PUT("/post/:post_id/favorite", putPostFavorite)
	authed.DELETE("/post/:post_id/favorite", deletePostFavorite)
	authed.PUT("/post/:post_id/bookmark", putPostBookmark)
	authed.DELETE("/post/:post_id/bookmark", deletePostBookmark)

	// Users
	unAuthed.GET("/user/:username/posts", getUserPosts)
	unAuthed.GET("/user/:username/followers", getUserFollowers)
	unAuthed.GET("/user/:username/following", getUserFollowing)
	authed.PUT("/user/:username/follow", followAUser)
	authed.DELETE("/user/:username/follow", unfollowAUser)

	// Timeline
	authed.GET("/timeline", getTimeline)

	e.Logger.Fatal(e.Start(":1323"))
}

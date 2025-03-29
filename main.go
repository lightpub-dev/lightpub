/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

package main

import (
	"flag"
	"log/slog"
	"net/http"
	"os"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/labstack/gommon/log"
	"github.com/lightpub-dev/lightpub/web"
	etag "github.com/pablor21/echo-etag/v4"
)

var (
	configFileFlag = flag.String("config", "config.yaml", "Path to the configuration file")
)

func main() {
	flag.Parse()

	slog.SetDefault(
		slog.New(slog.NewTextHandler(
			os.Stderr,
			&slog.HandlerOptions{},
		)),
	)

	e := echo.New()

	e.Use(middleware.LoggerWithConfig(middleware.LoggerConfig{
		Format: "method=${method}, status=${status}, uri=${uri}\n",
	}))
	e.Use(middleware.Recover())
	e.Use(errorHandleMiddleware)
	e.Use(etag.Etag())

	e.Renderer = web.TemplateRenderer
	e.Logger.SetLevel(log.DEBUG)

	s, err := web.NewStateFromConfigFile(*configFileFlag)
	if err != nil {
		e.Logger.Fatalf("Failed to create state: %v", err)
	}
	e.Debug = s.DevMode()

	authRequired := s.MakeJwtAuthMiddleware(false)
	authOptional := s.MakeJwtAuthMiddleware(true)

	e.GET("/", func(c echo.Context) error {
		return c.Redirect(http.StatusFound, "/client/timeline")
	})

	authGroup := e.Group("/auth")
	authGroup.POST("/register", s.RegisterUser)
	authGroup.POST("/login", s.LoginUser)
	authGroup.POST("/logout", s.LogoutUser, authOptional)

	noteGroup := e.Group("/note")
	noteGroup.POST("", s.CreateNote, authRequired)
	noteGroup.GET("/:id", s.GetNote, authOptional)
	noteGroup.DELETE("/:id", s.DeleteNote, authRequired)
	noteGroup.PATCH("/:id", s.PatchNote, authRequired)
	noteGroup.POST("/:id/renote", s.CreateRenote, authRequired)
	noteGroup.GET("/:id/replies", s.GetNoteReplies, authOptional)
	noteGroup.GET("/:id/renotes", s.GetRenotersModal, authOptional)
	noteGroup.GET("/:id/mentions", s.GetNoteMentionsModal, authOptional)
	noteGroup.PUT("/:id/bookmark", s.PutBookmarkOnNote, authRequired)
	noteGroup.DELETE("/:id/bookmark", s.DeleteBookmarkOnNote, authRequired)
	noteGroup.GET("/:id/edit", s.GetEditNotePage, authRequired)

	userGroup := e.Group("/user")
	userGroup.GET("/:id", s.ApubUser, web.CheckApubMiddleware)
	userGroup.PATCH("/:id", s.ProfileUpdate, authRequired)
	userGroup.GET("/:id/notes", s.GetUserNoteList, authOptional)
	userGroup.GET("/:id/avatar", s.GetUserAvatar)
	userGroup.GET("/:id/following", s.GetUserFollowings)
	userGroup.GET("/:id/followers", s.GetUserFollowers)
	userGroup.POST("/:id/interaction", s.UserInteraction, authRequired)
	userGroup.POST("/:id/inbox", s.Inbox, web.CheckApubMiddleware)

	notificationGroup := e.Group("/notification")
	notificationGroup.GET("/unread-count", s.GetUnreadNotificationCount, authRequired)
	notificationGroup.POST("/all/read", s.MarkAllNotificationsAsRead, authRequired)
	notificationGroup.GET("/all", s.GetNotifications, authRequired)
	notificationGroup.POST("/:id/read", s.MarkNotificationAsRead, authRequired)

	e.GET("/upload/:id", s.GetUpload)
	e.GET("/timeline", s.GetTimeline, authOptional)
	e.GET("/trends", s.GetTrends)

	e.POST("/inbox", s.Inbox, web.CheckApubMiddleware)

	e.GET("/.well-known/webfinger", s.WebFinger)

	clientGroup := e.Group("/client")
	clientGroup.GET("/register", s.ClientRegisterUser)
	clientGroup.GET("/login", s.ClientLoginUser)
	clientGroup.GET("/timeline", s.ClientTimeline, authOptional)
	clientGroup.GET("/my", s.ClientMy, authRequired)
	clientGroup.GET("/my/edit", s.ClientProfileUpdatePage, authRequired)
	clientGroup.GET("/user/:spec", s.ClientProfile, authOptional)
	clientGroup.GET("/user/:spec/following", s.ClientUserFollowings)
	clientGroup.GET("/user/:spec/followers", s.ClientUserFollowers)
	clientGroup.GET("/note/:id", s.ClientGetNote, authOptional)
	clientGroup.GET("/notification", s.ClientNotification, authRequired)

	e.GET("/healthcheck", func(c echo.Context) error {
		return c.String(http.StatusOK, "OK")
	})
	e.GET("/sw.js", func(c echo.Context) error {
		c.Response().Header().Set("Cache-Control", "public, max-age=86400")
		return c.File("static/js/sw.js")
	})

	e.Static("/static", "static")

	e.Logger.Fatal(e.Start(":8000"))
}

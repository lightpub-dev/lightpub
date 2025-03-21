package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/labstack/gommon/log"
	"github.com/lightpub-dev/lightpub/web"
)

func main() {
	e := echo.New()

	e.Use(middleware.Logger())
	e.Use(middleware.Recover())

	e.Renderer = templ
	e.Logger.SetLevel(log.DEBUG)

	s := web.State{}

	authRequired := s.MakeJwtAuthMiddleware(false)
	// authOptional := s.MakeJwtAuthMiddleware(true)

	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Hello, World!")
	})

	authGroup := e.Group("/auth")
	authGroup.POST("/register", s.RegisterUser)
	authGroup.POST("/login", s.LoginUser)
	authGroup.POST("/logout", s.LogoutUser, authRequired)

	clientGroup := e.Group("/client")
	clientGroup.GET("/register", s.ClientRegisterUser)

	e.Static("/static", "static")

	e.Logger.Fatal(e.Start(":8000"))
}

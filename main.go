package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/web"
)

func main() {
	e := echo.New()

	s := web.State{}

	authRequired := s.MakeJwtAuthMiddleware(false)
	authOptional := s.MakeJwtAuthMiddleware(true)

	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Hello, World!")
	})

	authGroup := e.Group("/auth")
	authGroup.POST("/register", s.RegisterUser)
	authGroup.POST("/login", s.LoginUser)
	authGroup.POST("/logout", s.LogoutUser, authRequired)

	e.Logger.Fatal(e.Start(":1323"))
}

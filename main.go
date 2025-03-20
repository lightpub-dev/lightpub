package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/web"
)

func main() {
	e := echo.New()

	authRequired := web.MakeJwtAuthMiddleware(false)
	authOptional := web.MakeJwtAuthMiddleware(true)

	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Hello, World!")
	})
	e.Logger.Fatal(e.Start(":1323"))
}

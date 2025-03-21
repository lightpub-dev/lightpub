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

	e.Renderer = templ
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
	noteGroup.GET("/:id", s.GetNote)
	noteGroup.POST("", s.CreateNote, authRequired)

	e.GET("/timeline", s.GetTimeline, authOptional)

	clientGroup := e.Group("/client")
	clientGroup.GET("/register", s.ClientRegisterUser)
	clientGroup.GET("/login", s.ClientLoginUser)
	clientGroup.GET("/timeline", s.ClientTimeline, authOptional)

	e.GET("/healthcheck", func(c echo.Context) error {
		return c.String(http.StatusOK, "OK")
	})

	e.Static("/static", "static")

	e.Logger.Fatal(e.Start(":8000"))
}

package main

import (
	"html/template"
	"io"
	"log"
	"net/http"
	"os"

	"github.com/labstack/echo/v4"
)

type Template struct {
	templates *template.Template
}

func (t *Template) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.templates.ExecuteTemplate(w, name, data)
}

type handler struct {
	req *Requester
}

func main() {
	e := echo.New()

	t := &Template{
		templates: template.Must(template.ParseGlob("templates/*.html")),
	}
	e.Renderer = t

	baseURL := os.Getenv("API_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:8000"
		log.Printf("API_BASE_URL is not set, using default: %s", baseURL)
	}

	h := &handler{
		req: NewRequester(baseURL, http.DefaultClient),
	}

	e.GET("/ping", func(c echo.Context) error {
		return c.String(http.StatusOK, "pong")
	})
	e.GET("/register", h.registerView)
	e.POST("/register", h.registerPost)
	e.GET("/login", h.loginView)
	e.POST("/login", h.loginPost)
	e.GET("/timeline", h.TimelineView)
	e.Logger.Fatal(e.Start(":1323"))
}

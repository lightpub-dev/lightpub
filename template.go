package main

import (
	"io"
	"text/template"

	"github.com/labstack/echo/v4"
)

var (
	templ = &Template{
		templates: template.Must(template.ParseGlob("templates/*.html")),
	}
)

type Template struct {
	templates *template.Template
}

func (t *Template) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.templates.ExecuteTemplate(w, name, data)
}

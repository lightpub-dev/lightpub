package web

import (
	"bytes"
	"html/template"
	"io"

	"github.com/labstack/echo/v4"
)

var (
	templates        = template.Must(template.ParseGlob("templates/*.html"))
	TemplateRenderer = &Template{
		templates: templates,
	}
)

type Template struct {
	templates *template.Template
}

func (t *Template) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.templates.ExecuteTemplate(w, name, data)
}

func renderTemplateToRawHTML(name string, data interface{}) template.HTML {
	var buf bytes.Buffer
	err := templates.ExecuteTemplate(&buf, name, data)
	if err != nil {
		return ""
	}
	return template.HTML(buf.String())
}

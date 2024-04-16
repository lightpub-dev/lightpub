package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

func (h *handler) TimelineView(c echo.Context) error {
	token := getToken(c)
	if token == "" {
		return c.Redirect(http.StatusSeeOther, "/login")
	}

	return c.Render(http.StatusOK, "timeline.html", nil)
}

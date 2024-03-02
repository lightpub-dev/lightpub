package api

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

func (h *Handler) UserInbox(c echo.Context) error {
	return c.NoContent(http.StatusMethodNotAllowed)
}

func (h *Handler) UserOutbox(c echo.Context) error {
	return c.NoContent(http.StatusMethodNotAllowed)
}

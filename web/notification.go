package web

import (
	"fmt"
	"net/http"

	"github.com/labstack/echo/v4"
)

func (s *State) GetUnreadNotificationCount(c echo.Context) error {
	viewerID := getViewerID(c)
	count, err := s.service.GetUnreadNotificationCount(c.Request().Context(), *viewerID)
	if err != nil {
		return err
	}

	if count == 0 {
		return c.String(http.StatusOK, "")
	}
	return c.String(http.StatusOK, fmt.Sprintf("%d", count))
}

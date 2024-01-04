package api

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/webfinger"
)

func (h *Handler) GetWebfinger(c echo.Context) error {
	// get resource parameter
	resource := c.QueryParam("resource")
	if resource == "" {
		return c.String(400, "invalid resource")
	}

	jsonResponse, err := webfinger.HandleWebfinger(c.Request().Context(), h.DB, resource)
	if err != nil {
		if err == webfinger.ErrBadFormat {
			return c.String(http.StatusBadRequest, "bad format")
		}
		if err == webfinger.ErrUnknown {
			return c.String(http.StatusUnprocessableEntity, "unknown")
		}
		if err == webfinger.ErrInvalidHost {
			return c.String(http.StatusUnprocessableEntity, "invalid host")
		}
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	return c.JSON(http.StatusOK, jsonResponse)
}

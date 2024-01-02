package main

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/timeline"
)

func getTimeline(c echo.Context) error {
	userID := c.Get(ContextUserID).(string)

	tl, err := timeline.FetchTimeline(c.Request().Context(), db, rdb, userID)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	return c.JSON(200, tl)
}

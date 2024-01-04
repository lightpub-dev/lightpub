package api

import (
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/timeline"
)

func (h *Handler) GetTimeline(c echo.Context) error {
	userID := c.Get(ContextUserID).(string)

	afterTimeStr := c.QueryParam("after_date")
	var afterTime *time.Time
	if afterTimeStr != "" {
		t, err := time.Parse(time.RFC3339, afterTimeStr)
		if err != nil {
			return c.String(400, "invalid after time")
		}
		afterTime = &t
	}

	beforeTimeStr := c.QueryParam("before_date")
	var beforeTime *time.Time
	if beforeTimeStr != "" {
		t, err := time.Parse(time.RFC3339, beforeTimeStr)
		if err != nil {
			return c.String(400, "invalid before time")
		}
		beforeTime = &t
	}

	limitStr := c.QueryParam("limit")
	var limit int
	if limitStr != "" {
		l, err := strconv.Atoi(limitStr)
		if err != nil {
			return c.String(400, "invalid limit")
		}
		limit = l
	}

	tl, err := timeline.FetchTimeline(c.Request().Context(), h.MakeDB(), userID, timeline.FetchOptions{
		AfterTime:  afterTime,
		BeforeTime: beforeTime,
		Limit:      limit,
	})
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	return c.JSON(200, tl)
}

package api

import (
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
)

func (h *Handler) GetTrend(c echo.Context) error {
	trendService := initializeTrendServices(c, h)
	trends, err := trendService.GetCurrentTrend()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(200, trends)
}

func (h *Handler) GetTrendPosts(c echo.Context) error {
	var viewerID db.UUID
	if c.Get(ContextAuthed).(bool) {
		viewerID = c.Get(ContextUserID).(db.UUID)
	}

	hashtag := c.QueryParam("hashtag")
	if hashtag == "" {
		return c.String(400, "hashtag not specified")
	}

	limitStr := c.QueryParam("limit")
	if limitStr == "" {
		limitStr = "10"
	}
	limit64, err := strconv.ParseInt(limitStr, 10, 64)
	if err != nil {
		return c.String(400, "invalid limit")
	}
	limit := int(limit64)

	var beforeDate *time.Time
	beforeDateStr := c.QueryParam("before_date")
	if beforeDateStr != "" {
		beforeDateT, err := time.Parse(time.RFC3339, beforeDateStr)
		if err != nil {
			return c.String(400, "invalid before_date")
		}
		beforeDate = &beforeDateT
	}

	trendService := initializeTrendServices(c, h)
	posts, err := trendService.GetTrendPosts(hashtag, viewerID, beforeDate, limit)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(200, posts)
}

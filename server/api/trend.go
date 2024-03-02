package api

/*
func (h *Handler) GetTrend(c echo.Context) error {
	trends, err := trend.GetCurrentTrend(c.Request().Context(), h.MakeDB())
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

	posts, err := trend.GetTrendPosts(c.Request().Context(), h.MakeDB(), hashtag, viewerID, beforeDate, limit)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(200, posts)
}
*/

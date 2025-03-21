package web

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

type ClientTimelineParams struct {
	TimelineURL string
	CreateNote  ClientCreateNoteParams
}

func (s *State) ClientTimeline(c echo.Context) error {
	isPublic := c.QueryParam("public") == "true"
	timelineURL := "/timeline"
	if isPublic {
		timelineURL += "?public=true"
	}

	params := ClientTimelineParams{
		TimelineURL: timelineURL,
		CreateNote: ClientCreateNoteParams{
			Authed: isAuthed(c),
			Title:  "ノート作成",
		},
	}
	return c.Render(http.StatusOK, "topTimeline.html", params)
}

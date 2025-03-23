/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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

	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "topTimeline.html", params)
}

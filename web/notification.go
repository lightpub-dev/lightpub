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
	"fmt"
	"html/template"
	"net/http"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/service/notification"
	"github.com/lightpub-dev/lightpub/types"
)

type NotificationListParams struct {
	Data    []NotificationParams
	NextURL string
}

type NotificationParams struct {
	ID        types.NotificationID
	IconURL   string
	Body      template.HTML
	CreatedAt time.Time
	IsRead    bool
}

type NotificationFollowedParams struct {
	FollowerURL      string
	FollowerNickname string
}

type NotificationFollowRequestedParams struct {
	FollowerURL      string
	FollowerNickname string
}

type NotificationFollowAcceptedParams struct {
	AcceptorURL      string
	AcceptorNickname string
}

type NotificationMentionedParams struct {
	AuthorURL      string
	AuthorNickname string
	NoteURL        string
}

type NotificationRenotedParams struct {
	AuthorURL      string
	AuthorNickname string
	TargetNoteURL  string
}

type NotificationRepliedParams struct {
	AuthorURL      string
	AuthorNickname string
	RepliedNoteURL string
	ReplyNoteURL   string
}

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

func (s *State) MarkNotificationAsRead(c echo.Context) error {
	notificationIDStr := c.Param("id")
	notificationID, err := types.ParseNotificationID(notificationIDStr)
	if err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "invalid notification ID")
	}

	viewerID := getViewerID(c)
	if err := s.service.ReadNotificationID(c.Request().Context(), *viewerID, notificationID); err != nil {
		return err
	}

	c.Response().Header().Set(hxRefresh, trueHeaderValue)
	return c.NoContent(http.StatusNoContent)
}

func (s *State) MarkAllNotificationsAsRead(c echo.Context) error {
	viewerID := getViewerID(c)
	if err := s.service.ReadAllNotifications(c.Request().Context(), *viewerID); err != nil {
		return err
	}

	c.Response().Header().Set(hxRefresh, trueHeaderValue)
	return c.NoContent(http.StatusNoContent)
}

func (s *State) ClientNotification(c echo.Context) error {
	return c.Render(http.StatusOK, "topNotification.html", nil)
}

func (s *State) GetNotifications(c echo.Context) error {
	viewerID := getViewerID(c)

	var param struct {
		Page int `query:"page"`
	}
	if err := c.Bind(&param); err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "invalid query parameter")
	}
	if param.Page < 0 {
		return echo.NewHTTPError(http.StatusBadRequest, "invalid query parameter")
	}

	ns, mayHaveNext, err := s.service.GetNotifications(c.Request().Context(), *viewerID, paginationSizeP1, int(param.Page))
	if err != nil {
		return err
	}

	nParams := make([]NotificationParams, 0, len(ns))
	for _, n := range ns {
		nParams = append(nParams, NotificationParams{
			ID:        n.ID,
			IconURL:   "", // TODO
			Body:      renderNotificationBody(n.Body),
			CreatedAt: n.CreatedAt,
			IsRead:    n.ReadAt != nil,
		})
	}

	nextURL := ""
	if mayHaveNext {
		nextURL = fmt.Sprintf("/notification?page=%d", param.Page+1)
	}

	return c.Render(http.StatusOK, "notification_list.html", NotificationListParams{
		Data:    nParams,
		NextURL: nextURL,
	})
}

func renderNotificationBody(body notification.Body) template.HTML {
	if body == nil {
		panic("notification body is nil")
	}

	switch b := body.(type) {
	case *notification.Followed:
		return renderTemplateToRawHTML("notification_followed.html", NotificationFollowedParams{
			FollowerURL:      "", // TODO
			FollowerNickname: b.FollowerUser.Nickname,
		})
	case *notification.FollowRequested:
		return renderTemplateToRawHTML("notification_follow_requested.html", NotificationFollowRequestedParams{
			FollowerURL:      "", // TODO
			FollowerNickname: b.RequesterUser.Nickname,
		})
	case *notification.FollowAccepted:
		return renderTemplateToRawHTML("notification_follow_accepted.html", NotificationFollowAcceptedParams{
			AcceptorURL:      "", // TODO
			AcceptorNickname: b.AcceptorUser.Nickname,
		})
	case *notification.Mentioned:
		return renderTemplateToRawHTML("notification_mentioned.html", NotificationMentionedParams{
			AuthorURL:      "", // TODO
			AuthorNickname: b.MentionerUser.Nickname,
			NoteURL:        "", // TODO
		})
	case *notification.Renote:
		return renderTemplateToRawHTML("notification_renoted.html", NotificationRenotedParams{
			AuthorURL:      "", // TODO
			AuthorNickname: b.RenoterUser.Nickname,
			TargetNoteURL:  "", // TODO
		})
	case *notification.Replied:
		return renderTemplateToRawHTML("notification_replied.html", NotificationRepliedParams{
			AuthorURL:      "", // TODO
			AuthorNickname: b.ReplierUser.Nickname,
			RepliedNoteURL: "", // TODO
			ReplyNoteURL:   "", // TODO
		})
	}

	panic(fmt.Sprintf("unknown notification body type: %T", body))
}

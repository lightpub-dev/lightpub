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
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/apub"
)

const (
	apubContentType = "application/activity+json"
	apubLdType      = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\""
)

func renderApubJson(c echo.Context, statusCode int, v interface{}) error {
	withContext := apub.WithContext(v)

	c.Response().Header().Set("Content-Type", apubLdType)

	return c.JSON(statusCode, withContext)
}

func CheckApubMiddleware(next echo.HandlerFunc) echo.HandlerFunc {
	return func(c echo.Context) error {
		if !containsApubAccept(c.Request().Header.Get("Accept")) {

			return c.String(406, "Not Acceptable")
		}

		return next(c)
	}
}

func containsApubAccept(acceptHeader string) bool {
	if strings.Contains(acceptHeader, apubContentType) {
		return true
	}
	if strings.Contains(acceptHeader, apubLdType) {
		return true
	}

	return false
}

// Inbox is a handler for both user-specific and shared inbox.
func (s *State) Inbox(c echo.Context) error {
	userID, err := s.Service().AuthorizeWithHttpSig(c.Request().Context(), c.Request())
	if err != nil {
		return err
	}

	body := c.Request().Body
	defer body.Close()

	activity, err := apub.ParseActivity(body)
	if err != nil {
		return err
	}

	if err := s.Service().ReceiveActivity(c.Request().Context(), userID, activity); err != nil {
		return err
	}

	return c.NoContent(http.StatusOK)
}

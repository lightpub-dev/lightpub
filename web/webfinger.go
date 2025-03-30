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
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/types"
)

type webFingerData struct {
	Subject string          `json:"subject"`
	Aliases []string        `json:"aliases,omitempty"`
	Links   []webFingerLink `json:"links"`
}

type webFingerLink struct {
	Rel  string `json:"rel"`
	Type string `json:"type"`
	Href string `json:"href"`
}

func (s *State) WebFinger(c echo.Context) error {
	resource := c.QueryParam("resource")
	if resource == "" {
		return c.String(400, "missing resource")
	}

	usernameAndDomain, found := strings.CutPrefix(resource, "acct:")
	if !found {
		return c.String(400, "invalid resource")
	}

	parts := strings.SplitN(usernameAndDomain, "@", 2)
	if len(parts) != 2 {
		return c.String(400, "invalid resource")
	}

	username := parts[0]
	domain := parts[1]
	if domain != s.MyDomain() {
		return c.String(404, "user not on this server")
	}

	spec := fmt.Sprintf("@%s@%s", username, domain)
	specifier, ok := types.ParseUserSpecifier(spec, s.MyDomain())
	if !ok {
		return c.String(400, "invalid resource")
	}

	userID, err := s.Service().FindUserIDBySpecifierWithRemote(c.Request().Context(), specifier)
	if err != nil {
		return err
	}
	if userID == nil {
		return c.String(404, "user not found")
	}

	user, err := s.Service().FindApubUserByID(c.Request().Context(), *userID)
	if err != nil {
		return err
	}
	if user == nil {
		return c.String(404, "user not found")
	}

	return c.JSON(200, webFingerData{
		Subject: fmt.Sprintf("acct:%s:%s", username, domain),
		Links: []webFingerLink{
			{
				Rel:  "http://webfinger.net/rel/profile-page",
				Type: "text/html",
				Href: user.Apub.ViewURL,
			},
			{
				Rel:  "self",
				Type: apubLdType,
				Href: user.Apub.URL,
			},
		},
	})
}

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
	"errors"
	"net/http"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/types"
)

var (
	errBadInput = failure.NewError(http.StatusBadRequest, "invalid request")
)

const (
	jwtCookieName = "auth_token"
	authCtxName   = "user"
)

type authedUser struct {
	UserID types.UserID
}

func (s *State) MakeJwtAuthMiddleware(authIsOptional bool) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			token, err := getCookieFromReq(c)
			if err != nil {
				return err
			}

			if token == "" {
				if authIsOptional {
					return next(c)
				} else {
					return failure.NewError(http.StatusUnauthorized, "missing auth")
				}
			}

			claims, err := s.auth.VerifyJWT(token)
			if err != nil {
				return err
			}

			userID, err := types.ParseUserID(claims.Sub)
			if err != nil {
				return failure.NewError(http.StatusUnauthorized, "invalid user ID")
			}

			valid, err := s.service.CheckUserLoginExpiration(c.Request().Context(), userID, time.Unix(claims.Iat, 0))
			if err != nil {
				return err
			}
			if !valid {
				return failure.NewError(http.StatusUnauthorized, "login expired")
			}

			// Store authenticated user in context
			c.Set(authCtxName, authedUser{UserID: userID})

			return next(c)
		}
	}
}

func getCookieFromReq(c echo.Context) (string, error) {
	// Implementation depends on how you're storing tokens
	// Either from cookies or Authorization header
	token, err := c.Cookie(jwtCookieName)
	if err != nil {
		if errors.Is(err, http.ErrNoCookie) {
			return "", nil
		}
		return "", err
	}
	if token != nil && token.Value != "" {
		return token.Value, nil
	}

	return "", nil
}

func isAuthed(c echo.Context) bool {
	_, ok := c.Get(authCtxName).(authedUser)
	return ok
}

func getAuth(c echo.Context) (authedUser, bool) {
	auth, ok := c.Get(authCtxName).(authedUser)
	return auth, ok
}

func getViewerID(c echo.Context) *types.UserID {
	auth, ok := getAuth(c)
	if !ok {
		return nil
	}
	return &auth.UserID
}

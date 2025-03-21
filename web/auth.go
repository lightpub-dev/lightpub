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

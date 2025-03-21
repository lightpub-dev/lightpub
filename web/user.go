package web

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/service"
)

const (
	jwtCookieAge = 24 * 60 * 60 // 24 hours
)

func (s *State) RegisterUser(c echo.Context) error {
	if !s.registrationOpen {
		return failure.NewError(http.StatusBadRequest, "registration is closed")
	}

	var req struct {
		Username string `json:"username" validate:"required,min=3,max=64"`
		Nickname string `json:"nickname" validate:"required,min=1,max=128"`
		Password string `json:"password" validate:"required,min=8,max=64"`
	}
	if err := c.Bind(&req); err != nil {
		return errBadInput
	}
	if err := validate.Struct(&req); err != nil {
		return err
	}

	userID, err := s.service.CreateNewLocalUser(c.Request().Context(), service.UserCreateParams{
		Username: req.Username,
		Nickname: req.Nickname,
		Password: req.Password,
	})
	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRedirect, "/client/login")
	return c.JSON(http.StatusOK, map[string]interface{}{
		"user_id": userID,
	})
}

func (s *State) LoginUser(c echo.Context) error {
	var req struct {
		Username string `json:"username" validate:"required"`
		Password string `json:"password" validate:"required"`
	}
	if err := c.Bind(&req); err != nil {
		return errBadInput
	}
	if err := validate.Struct(&req); err != nil {
		return err
	}

	userID, err := s.service.LoginUser(c.Request().Context(), req.Username, req.Password)
	if err != nil {
		return err
	}
	if userID == nil {
		return failure.NewError(http.StatusUnauthorized, "invalid username or password")
	}

	token, err := s.auth.GenerateJWT(*userID)
	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRedirect, "/client/timeline")
	c.SetCookie(&http.Cookie{
		Name:     jwtCookieName,
		Value:    token,
		SameSite: http.SameSiteLaxMode,
		Secure:   !s.DevMode(),
		HttpOnly: true,
		MaxAge:   jwtCookieAge,
	})
	return c.JSON(http.StatusOK, map[string]interface{}{
		"user_id": userID,
		"token":   token,
	})
}

func (s *State) LogoutUser(c echo.Context) error {
	type query struct {
		All *bool `query:"all"`
	}

	var q query
	if err := c.Bind(&q); err != nil {
		return errBadInput
	}

	c.SetCookie(&http.Cookie{
		Name:     jwtCookieName,
		SameSite: http.SameSiteLaxMode,
		Secure:   !s.DevMode(),
		HttpOnly: true,
		MaxAge:   -1, // delete cookie
	})

	if q.All != nil && *q.All {
		if err := s.service.LogoutAllUser(c.Request().Context(), c.Get(authCtxName).(*authedUser).UserID); err != nil {
			return err
		}
	}

	c.Response().Header().Set(hxRedirect, "/client/login")
	return c.JSON(http.StatusOK, nil)
}

func (s *State) ClientRegisterUser(c echo.Context) error {
	return c.Render(http.StatusOK, "topRegister.html", nil)
}

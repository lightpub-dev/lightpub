package web

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/service"
)

func (s *State) RegisterUser(c echo.Context) error {
	if !s.registrationOpen {
		return failure.NewError(http.StatusBadRequest, "registration is closed")
	}

	var req struct {
		Username string `json:"username",validate:"required,min=3,max=64"`
		Nickname string `json:"nickname",validate:"required,min=1,max=128"`
		Password string `json:"password",validate:"required,min=8,max=64"`
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

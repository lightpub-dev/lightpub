package web

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/apub"
	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) ApubUser(c echo.Context) error {
	id := c.Param("id")
	userID, err := types.ParseUserID(id)
	if err != nil {
		return errBadInput
	}

	u, err := s.Service().FindApubUserByID(c.Request().Context(), userID)
	if err != nil {
		return err
	}
	if u == nil {
		return c.NoContent(http.StatusNotFound)
	}

	aUser, err := apub.NewUser(*u)
	if err != nil {
		return err
	}

	return renderApubJson(c, http.StatusOK, aUser)
}

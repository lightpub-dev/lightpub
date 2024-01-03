package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/webfinger"
)

func getWebfinger(c echo.Context) error {
	// get resource parameter
	resource := c.QueryParam("resource")
	if resource == "" {
		return c.String(400, "invalid resource")
	}

	jsonResponse, err := webfinger.HandleWebfinger(resource)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	return c.JSON(http.StatusOK, jsonResponse)
}

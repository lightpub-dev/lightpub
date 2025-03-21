package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
)

type errorResponse struct {
	Message string `json:"message"`
}

func errorHandleMiddleware(next echo.HandlerFunc) echo.HandlerFunc {
	return func(c echo.Context) error {
		err := next(c)
		if err != nil {
			e, ok := err.(failure.ErrorResponse)
			if ok {
				if e.StatusCode() == http.StatusInternalServerError {
					c.Logger().Errorf("ise: %v", e)
				}

				return c.JSON(e.StatusCode(), errorResponse{Message: e.Message()})
			}

			c.Logger().Errorf("%v", err)
		}
		return err
	}
}

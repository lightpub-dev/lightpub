package main

import (
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
				return c.JSON(e.StatusCode(), errorResponse{Message: e.Message()})
			}
		}
		return err
	}
}

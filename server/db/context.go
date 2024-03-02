package db

import (
	"context"

	"github.com/labstack/echo/v4"
)

type Context struct {
	Ctx context.Context
}

func ProvideContext(c echo.Context) Context {
	return Context{Ctx: c.Request().Context()}
}

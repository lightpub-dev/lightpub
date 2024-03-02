//go:build wireinject
// +build wireinject

package api

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"

	"github.com/google/wire"
	"github.com/lightpub-dev/lightpub/users"
)

func ProvideDBConnFromHandler(ctx db.Context, h *Handler) db.DBConn {
	return db.DBConn{DB: h.DB, Ctx: ctx}
}

func initializeUserCreateService(c echo.Context, h *Handler) *users.DBUserCreateService {
	wire.Build(db.ProvideContext, ProvideDBConnFromHandler, users.ProvideDBUserCreateService)
	return &users.DBUserCreateService{}
}

func initializeUserLoginService(c echo.Context, h *Handler) *users.DBUserLoginService {
	wire.Build(db.ProvideContext, ProvideDBConnFromHandler, users.ProvideDBUserLoginService)
	return &users.DBUserLoginService{}
}

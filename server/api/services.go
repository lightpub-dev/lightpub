//go:build wireinject
// +build wireinject

package api

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"

	"github.com/google/wire"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/timeline"
	"github.com/lightpub-dev/lightpub/users"
)

func ProvideDBConnFromHandler(ctx db.Context, h *Handler) db.DBConn {
	return db.DBConn{DB: h.DB, Ctx: ctx}
}

func initializeUserCreateService(c echo.Context, h *Handler) users.UserCreateService {
	wire.Build(
		db.ProvideContext,
		ProvideDBConnFromHandler,
		users.ProvideDBUserCreateService,
		wire.Bind(
			new(users.UserCreateService), new(*users.DBUserCreateService),
		),
	)
	return nil
}

func initializeUserLoginService(c echo.Context, h *Handler) users.UserLoginService {
	wire.Build(
		db.ProvideContext,
		ProvideDBConnFromHandler,
		users.ProvideDBUserLoginService,
		wire.Bind(
			new(users.UserLoginService), new(*users.DBUserLoginService),
		),
	)
	return nil
}

func initializeTimelineService(c echo.Context, h *Handler) timeline.TimelineService {
	wire.Build(db.ProvideContext,
		ProvideDBConnFromHandler,
		wire.Bind(
			new(users.UserFollowService), new(*users.DBUserFollowService),
		),
		users.ProvideDBUserFollowService,
		posts.DBPostServices,
		wire.Bind(
			new(timeline.TimelineService),
			new(*timeline.DBTimelineService),
		),
		timeline.ProvideDBTimelineService,
	)
	return nil
}

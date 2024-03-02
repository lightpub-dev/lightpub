//go:build wireinject
// +build wireinject

package api

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"

	"github.com/google/wire"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/reactions"
	"github.com/lightpub-dev/lightpub/timeline"
	"github.com/lightpub-dev/lightpub/trend"
	"github.com/lightpub-dev/lightpub/users"
)

func ProvideDBConnFromHandler(ctx db.Context, h *Handler) db.DBConn {
	return db.DBConn{DB: h.DB, Ctx: ctx}
}

var (
	DBSet = wire.NewSet(
		ProvideDBConnFromHandler,
		db.ProvideContext,
	)
)

func initializeUserCreateService(c echo.Context, h *Handler) users.UserCreateService {
	wire.Build(
		DBSet,
		users.ProvideDBUserCreateService,
		wire.Bind(
			new(users.UserCreateService), new(*users.DBUserCreateService),
		),
	)
	return nil
}

func initializeUserLoginService(c echo.Context, h *Handler) users.UserLoginService {
	wire.Build(
		DBSet,
		users.ProvideDBUserLoginService,
		wire.Bind(
			new(users.UserLoginService), new(*users.DBUserLoginService),
		),
	)
	return nil
}

func initializeTimelineService(c echo.Context, h *Handler) timeline.TimelineService {
	wire.Build(
		DBSet,
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

func initializePostCreateService(c echo.Context, h *Handler) posts.PostCreateService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
	)
	return nil
}

func initializePostReactionService(c echo.Context, h *Handler) posts.PostReactionService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		reactions.DBReactionServices,
		posts.DBPostServices,
	)
	return nil
}

func initializePostLikeService(c echo.Context, h *Handler) posts.PostLikeService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
	)
	return nil
}

func initializePostFetchService(c echo.Context, h *Handler) posts.PostFetchService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
	)
	return nil
}

func initializeTrendServices(c echo.Context, h *Handler) trend.TrendService {
	wire.Build(
		DBSet,
		posts.DBPostServices,
		trend.DBTrendServices,
	)
	return nil
}

//go:build wireinject
// +build wireinject

package api

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"

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
	PubSet = wire.NewSet(
		pub.PubServices,
		ProvideIDGetter,
		wire.Bind(new(pub.IDGetterService), new(*IDGetter)),
		ProvideGoRequesterService,
		wire.Bind(new(pub.RequesterService), new(*pub.GoRequesterService)),
	)
)

func initializeUserCreateService(c echo.Context, h *Handler) users.UserCreateService {
	wire.Build(
		DBSet,
		users.DBUserServices,
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
		users.DBUserServices,
		posts.DBPostServices,
		PubSet,
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
		PubSet,
	)
	return nil
}

func initializePostReactionService(c echo.Context, h *Handler) posts.PostReactionService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		reactions.DBReactionServices,
		posts.DBPostServices,
		PubSet,
	)
	return nil
}

func initializePostLikeService(c echo.Context, h *Handler) posts.PostLikeService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
		PubSet,
	)
	return nil
}

func initializePostFetchService(c echo.Context, h *Handler) posts.PostFetchService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
		PubSet,
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

func initializeUserFinderService(c echo.Context, h *Handler) users.UserFinderService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		PubSet,
	)
	return nil
}

func initializeUserFollowService(c echo.Context, h *Handler) users.UserFollowService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		PubSet,
	)
	return nil
}

func initializeUserProfileService(c echo.Context, h *Handler) users.UserProfileService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		PubSet,
	)
	return nil
}

func initializeUserPostService(c echo.Context, h *Handler) posts.UserPostService {
	wire.Build(
		DBSet,
		users.DBUserServices,
		posts.DBPostServices,
		PubSet,
	)
	return nil
}

func initializePostCountService(c echo.Context, h *Handler) posts.PostCountService {
	wire.Build(
		DBSet,
		posts.DBPostServices,
	)
	return nil
}

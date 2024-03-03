// Code generated by Wire. DO NOT EDIT.

//go:generate go run -mod=mod github.com/google/wire/cmd/wire
//go:build !wireinject
// +build !wireinject

package api

import (
	"github.com/google/wire"
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/pub"
	"github.com/lightpub-dev/lightpub/reactions"
	"github.com/lightpub-dev/lightpub/timeline"
	"github.com/lightpub-dev/lightpub/trend"
	"github.com/lightpub-dev/lightpub/users"
)

// Injectors from services.go:

func initializeUserCreateService(c echo.Context, h *Handler) users.UserCreateService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	return dbUserCreateService
}

func initializeUserLoginService(c echo.Context, h *Handler) users.UserLoginService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbUserLoginService := users.ProvideDBUserLoginService(dbConn)
	return dbUserLoginService
}

func initializeTimelineService(c echo.Context, h *Handler) timeline.TimelineService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbPostInteractionService := posts.ProvideDBPostInteractionService(dbConn)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbPostVisibilityService := posts.ProvideDBPostVisibilityService(dbConn, dbUserFollowService)
	dbPostFetchService := posts.ProvideDBPostFetchService(dbConn, dbPostVisibilityService, dbPostCountService)
	dbTimelineService := timeline.ProvideDBTimelineService(dbConn, dbPostInteractionService, dbPostCountService, dbPostFetchService)
	return dbTimelineService
}

func initializePostCreateService(c echo.Context, h *Handler) posts.PostCreateService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbPostVisibilityService := posts.ProvideDBPostVisibilityService(dbConn, dbUserFollowService)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	dbPostFetchService := posts.ProvideDBPostFetchService(dbConn, dbPostVisibilityService, dbPostCountService)
	dbPostCreateService := posts.ProvideDBPostCreateService(dbConn, dbPostVisibilityService, dbPostFetchService)
	return dbPostCreateService
}

func initializePostReactionService(c echo.Context, h *Handler) posts.PostReactionService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbFindReactionService := reactions.ProvideDBFindReactionService(dbConn)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbPostVisibilityService := posts.ProvideDBPostVisibilityService(dbConn, dbUserFollowService)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	dbPostFetchService := posts.ProvideDBPostFetchService(dbConn, dbPostVisibilityService, dbPostCountService)
	dbPostReactionService := posts.ProvideDBPostReactionService(dbConn, dbFindReactionService, dbPostVisibilityService, dbPostFetchService)
	return dbPostReactionService
}

func initializePostLikeService(c echo.Context, h *Handler) posts.PostLikeService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbPostVisibilityService := posts.ProvideDBPostVisibilityService(dbConn, dbUserFollowService)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	dbPostFetchService := posts.ProvideDBPostFetchService(dbConn, dbPostVisibilityService, dbPostCountService)
	dbPostLikeService := posts.ProvideDBPostLikeService(dbConn, dbPostVisibilityService, dbPostFetchService)
	return dbPostLikeService
}

func initializePostFetchService(c echo.Context, h *Handler) posts.PostFetchService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbPostVisibilityService := posts.ProvideDBPostVisibilityService(dbConn, dbUserFollowService)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	dbPostFetchService := posts.ProvideDBPostFetchService(dbConn, dbPostVisibilityService, dbPostCountService)
	return dbPostFetchService
}

func initializeTrendServices(c echo.Context, h *Handler) trend.TrendService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	dbTrendService := trend.ProvideDBTrendService(dbConn, dbPostCountService)
	return dbTrendService
}

func initializeUserFinderService(c echo.Context, h *Handler) users.UserFinderService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	return dbUserFinderService
}

func initializeUserFollowService(c echo.Context, h *Handler) users.UserFollowService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	return dbUserFollowService
}

func initializeUserProfileService(c echo.Context, h *Handler) users.UserProfileService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserProfileService := users.ProvideDBUserProfileService(dbConn, dbUserFinderService)
	return dbUserProfileService
}

func initializeUserPostService(c echo.Context, h *Handler) posts.UserPostService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbPostInteractionService := posts.ProvideDBPostInteractionService(dbConn)
	idGetter := ProvideIDGetter(h)
	dbKeyResolveService := ProvideDBKeyResolveService(dbConn, idGetter)
	signatureService := pub.ProvideSignatureService(dbKeyResolveService)
	goRequesterService := ProvideGoRequesterService(h, signatureService)
	pubFollowService := users.ProvidePubFollowService(idGetter, goRequesterService)
	dbUserKeyService := users.ProvideDBUserKeyService(dbConn)
	dbUserCreateService := users.ProvideDBUserCreateService(dbConn, dbUserKeyService)
	webfingerService := pub.ProvideWebfingerService(goRequesterService)
	pubUserService := users.ProvidePubUserService(dbUserCreateService, goRequesterService, webfingerService)
	dbUserFinderService := users.ProvideDBUserFinder(dbConn, pubUserService, idGetter)
	dbUserFollowService := users.ProvideDBUserFollowService(dbConn, pubFollowService, dbUserFinderService, idGetter)
	dbUserPostService := posts.ProvideDBUserPostService(dbConn, dbPostInteractionService, dbUserFollowService)
	return dbUserPostService
}

func initializePostCountService(c echo.Context, h *Handler) posts.PostCountService {
	context := db.ProvideContext(c)
	dbConn := ProvideDBConnFromHandler(context, h)
	dbPostCountService := posts.ProvideDBPostCountService(dbConn)
	return dbPostCountService
}

func initializePubUserService(c echo.Context, h *Handler) *pub.PubUserService {
	idGetter := ProvideIDGetter(h)
	pubUserService := pub.ProvidePubUserService(idGetter)
	return pubUserService
}

// services.go:

func ProvideDBConnFromHandler(ctx db.Context, h *Handler) db.DBConn {
	return db.DBConn{DB: h.DB, Ctx: ctx}
}

var (
	DBSet = wire.NewSet(
		ProvideDBConnFromHandler, db.ProvideContext,
	)
	PubSet = wire.NewSet(pub.PubServices, ProvideIDGetter, wire.Bind(new(pub.IDGetterService), new(*IDGetter)), ProvideGoRequesterService, wire.Bind(new(pub.RequesterService), new(*pub.GoRequesterService)), ProvideDBKeyResolveService, wire.Bind(new(pub.KeyResolveService), new(*DBKeyResolveService)))
)

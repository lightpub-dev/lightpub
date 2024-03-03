package users

import "github.com/google/wire"

var (
	DBUserServices = wire.NewSet(
		ProvideDBUserCreateService,
		ProvideDBUserLoginService,
		ProvideDBUserFollowService,
		ProvideDBUserFinder,
		ProvideDBUserProfileService,
		ProvidePubFollowService,
		ProvidePubUserService,
		wire.Bind(
			new(UserCreateService), new(*DBUserCreateService),
		),
		wire.Bind(
			new(UserLoginService), new(*DBUserLoginService),
		),
		wire.Bind(
			new(UserFollowService), new(*DBUserFollowService),
		),
		wire.Bind(
			new(UserFinderService), new(*DBUserFinderService),
		),
		wire.Bind(
			new(UserProfileService), new(*DBUserProfileService),
		),
	)
)

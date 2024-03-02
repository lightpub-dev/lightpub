package posts

import "github.com/google/wire"

var (
	DBPostServices = wire.NewSet(
		ProvideDBPostCountService,
		ProvideDBPostCreateService,
		ProvideDBPostFetchService,
		ProvideDBPostInteractionService,
		ProvideDBPostVisibilityService,
		ProvideDBPostReactionService,
		ProvideDBPostLikeService,
		wire.Bind(
			new(PostCountService), new(*DBPostCountService),
		),
		wire.Bind(
			new(PostCreateService), new(*DBPostCreateService),
		),
		wire.Bind(
			new(PostFetchService), new(*DBPostFetchService),
		),
		wire.Bind(
			new(PostInteractionService), new(*DBPostInteractionService),
		),
		wire.Bind(
			new(PostVisibilityService), new(*DBPostVisibilityService),
		),
		wire.Bind(
			new(PostReactionService), new(*DBPostReactionService),
		),
		wire.Bind(
			new(PostLikeService), new(*DBPostLikeService),
		),
	)
)

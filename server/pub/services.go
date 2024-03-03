package pub

import "github.com/google/wire"

var (
	PubServices = wire.NewSet(
		ProvidePubUserService,
		ProvideWebfingerService,
	)
)

package pub

import "github.com/google/wire"

var (
	PubServices = wire.NewSet(
		ProvidePubUserService,
		ProvideGoRequesterService,
		wire.Bind(new(RequesterService), new(*GoRequesterService)),
	)
)

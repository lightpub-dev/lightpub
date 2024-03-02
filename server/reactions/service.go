package reactions

import "github.com/google/wire"

var (
	DBReactionServices = wire.NewSet(
		ProvideDBFindReactionService,
		wire.Bind(
			new(FindReactionService), new(*DBFindReactionService),
		),
	)
)

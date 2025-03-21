package service

import (
	"context"

	"github.com/lightpub-dev/lightpub/types"
)

type calculateToAndCcResult struct {
	To      []string
	Cc      []string
	Inboxes []string
}

func (s *State) calculateToAndCc(ctx context.Context, noteID types.NoteID, authorID types.UserID, visibility types.NoteVisibility, includeAuthor bool) {

}

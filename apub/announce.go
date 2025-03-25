package apub

import "github.com/lightpub-dev/lightpub/types"

type AnnounceActivity struct {
	ID     string   `json:"id" validate:"required"`
	Kind   string   `json:"type" validate:"required"`
	Actor  URI      `json:"actor" validate:"required"`
	To     []string `json:"to"`
	Cc     []string `json:"cc"`
	Object ObjectID `json:"object" validate:"required"`
}

func (a *AnnounceActivity) InferredVisibility() types.NoteVisibility {
	return inferVisibility(a.To, a.Cc)
}

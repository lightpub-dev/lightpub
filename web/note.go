package web

type ClientCreateNoteParams struct {
	Authed    bool
	Title     string
	ReplyToID *string
}

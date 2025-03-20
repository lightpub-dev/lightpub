package types

import "time"

const (
	NoteContentTypePlain NoteContentType = "plain"
	NoteContentTypeHTML  NoteContentType = "html"
	NoteContentTypeMD    NoteContentType = "md"
	NoteContentTypeLatex NoteContentType = "latex"

	NoteVisibilityPublic   NoteVisibility = "public"
	NoteVisibilityUnlisted NoteVisibility = "unlisted"
	NoteVisibilityFollower NoteVisibility = "follower"
	NoteVisibilityPrivate  NoteVisibility = "private"
)

type NoteContentType string
type NoteVisibility string

func (n NoteVisibility) ValidAsRenote() bool {
	return n == NoteVisibilityPublic || n == NoteVisibilityUnlisted
}

func (n NoteVisibility) AcceptRenote() bool {
	return n == NoteVisibilityPublic || n == NoteVisibilityUnlisted
}

type SimpleNote struct {
	ID         NoteID
	Author     NoteAuthor
	Content    *NoteContent
	Visibility NoteVisibility
	CreatedAt  time.Time
	UpdatedAt  *time.Time

	ReplyToID  *NoteID
	RenoteOfID *NoteID
	DeletedAt  *time.Time

	Sensitive bool
	Uploads   []UploadID
}

type NoteAuthor struct {
	ID       UserID
	Username string
	Nickname string
	Domain   string
}

func (n NoteAuthor) IsLocal() bool {
	return n.Domain == EmptyDomain
}

func (n NoteAuthor) IsRemote() bool {
	return n.Domain != EmptyDomain
}

type NoteContent struct {
	Type NoteContentType
	Data string
}

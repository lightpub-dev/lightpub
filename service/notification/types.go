package notification

import (
	"encoding/json"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

type Notification struct {
	ID        types.NotificationID
	UserID    types.UserID
	Body      Body // NotificationBody
	CreatedAt time.Time
	ReadAt    *time.Time // nil means unread
}

type NoteData struct {
	NoteID  types.NoteID
	ViewURL string
}

type Body interface {
	NotificationBody()
}

type Followed struct {
	FollowerUserID types.UserID `json:"f"`

	FollowerUser *types.SimpleUser `json:"-"`
}

func (*Followed) NotificationBody() {}

type FollowRequested struct {
	RequesterUserID types.UserID `json:"r"`

	RequesterUser *types.SimpleUser `json:"-"`
}

func (*FollowRequested) NotificationBody() {}

type FollowAccepted struct {
	AcceptorUserID types.UserID `json:"a"`

	AcceptorUser *types.SimpleUser `json:"-"`
}

func (*FollowAccepted) NotificationBody() {}

type Replied struct {
	ReplierUserID types.UserID `json:"a"`
	ReplyNoteID   types.NoteID `json:"r"`
	RepliedNoteID types.NoteID `json:"n"`

	ReplierUser *types.SimpleUser `json:"-"`
	ReplyNote   *NoteData         `json:"-"`
	RepliedNote *NoteData         `json:"-"`
}

func (*Replied) NotificationBody() {}

type Mentioned struct {
	MentionerUserID types.UserID `json:"m"`
	MentionNoteID   types.NoteID `json:"n"`

	MentionerUser *types.SimpleUser `json:"-"`
	MentionNote   *NoteData         `json:"-"`
}

func (*Mentioned) NotificationBody() {}

type Renote struct {
	RenoterUserID types.UserID `json:"r"`
	RenoteNoteID  types.NoteID `json:"n"`

	RenoterUser *types.SimpleUser `json:"-"`
}

func (*Renote) NotificationBody() {}

func Stringify(b Body) (string, error) {
	bodyJson, err := json.Marshal(b)
	if err != nil {
		return "", err
	}
	return string(bodyJson), nil
}

func ParseBody(s string) (Body, error) {
	var b Body
	if err := json.Unmarshal([]byte(s), &b); err != nil {
		return nil, err
	}
	return b, nil
}

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

type Body interface {
	NotificationBody()
}

type Followed struct {
	FollowerUserID types.UserID `json:"f"`
}

func (Followed) NotificationBody() {}

type FollowRequested struct {
	RequesterUserID types.UserID `json:"r"`
}

func (FollowRequested) NotificationBody() {}

type FollowAccepted struct {
	AcceptorUserID types.UserID `json:"a"`
}

func (FollowAccepted) NotificationBody() {}

type Replied struct {
	ReplierUserID types.UserID `json:"a"`
	ReplyNoteID   types.NoteID `json:"r"`
	RepliedNoteID types.NoteID `json:"n"`
}

func (Replied) NotificationBody() {}

type Mentioned struct {
	MentionerUserID types.UserID `json:"m"`
	MentionNoteID   types.NoteID `json:"n"`
}

func (Mentioned) NotificationBody() {}

type Renote struct {
	RenoterUserID types.UserID `json:"r"`
	RenoteNoteID  types.NoteID `json:"n"`
}

func (Renote) NotificationBody() {}

func Stringify(b Body) (string, error) {
	bodyJson, err := json.Marshal(b)
	if err != nil {
		return "", err
	}
	return string(bodyJson), nil
}

package notification

import (
	"encoding/json"
	"fmt"
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
	AsDB() (string, error)
}

func toJSON(t any) (string, error) {
	b, err := json.Marshal(t)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

type Followed struct {
	FollowerUserID types.UserID `json:"f"`

	FollowerUser *types.SimpleUser `json:"-"`
}

func (f *Followed) AsDB() (string, error) {
	return toJSON(*f)
}

type FollowRequested struct {
	RequesterUserID types.UserID `json:"r"`

	RequesterUser *types.SimpleUser `json:"-"`
}

func (f *FollowRequested) AsDB() (string, error) {
	return toJSON(*f)
}

type FollowAccepted struct {
	AcceptorUserID types.UserID `json:"a"`

	AcceptorUser *types.SimpleUser `json:"-"`
}

func (f *FollowAccepted) AsDB() (string, error) {
	return toJSON(*f)
}

type Replied struct {
	ReplierUserID types.UserID `json:"a"`
	ReplyNoteID   types.NoteID `json:"r"`
	RepliedNoteID types.NoteID `json:"n"`

	ReplierUser *types.SimpleUser `json:"-"`
	ReplyNote   *NoteData         `json:"-"`
	RepliedNote *NoteData         `json:"-"`
}

func (r *Replied) AsDB() (string, error) {
	return toJSON(*r)
}

type Mentioned struct {
	MentionerUserID types.UserID `json:"m"`
	MentionNoteID   types.NoteID `json:"n"`

	MentionerUser *types.SimpleUser `json:"-"`
	MentionNote   *NoteData         `json:"-"`
}

func (m *Mentioned) AsDB() (string, error) {
	return toJSON(*m)
}

type Renote struct {
	RenoterUserID types.UserID `json:"r"`
	RenoteNoteID  types.NoteID `json:"n"`

	RenoterUser *types.SimpleUser `json:"-"`
}

func (r *Renote) AsDB() (string, error) {
	return toJSON(*r)
}

func ParseBody(s string) (Body, error) {
	b := []byte(s)

	var follow Followed
	if err := json.Unmarshal(b, &follow); err == nil {
		return &follow, nil
	}

	var followRequested FollowRequested
	if err := json.Unmarshal(b, &followRequested); err != nil {
		return &followRequested, nil
	}

	var followAccepted FollowAccepted
	if err := json.Unmarshal(b, &followAccepted); err != nil {
		return &followAccepted, nil
	}

	var replied Replied
	if err := json.Unmarshal(b, &replied); err != nil {
		return &replied, nil
	}

	var mentioned Mentioned
	if err := json.Unmarshal(b, &mentioned); err != nil {
		return &mentioned, nil
	}

	var renote Renote
	if err := json.Unmarshal(b, &renote); err != nil {
		return &renote, nil
	}

	return nil, fmt.Errorf("unknown notification body type: %s", s)
}

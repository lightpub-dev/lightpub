package types

import (
	"github.com/google/uuid"
	"github.com/oklog/ulid/v2"
)

type UserID = ulid.ULID
type NoteID = ulid.ULID
type NotificationID = int32
type UploadID = uuid.UUID

func NewUserID() UserID {
	return ulid.Make()
}

func NewNoteID() NoteID {
	return ulid.Make()
}

func NewUploadID() UploadID {
	return uuid.New()
}

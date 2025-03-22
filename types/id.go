package types

import (
	"github.com/google/uuid"
	"github.com/oklog/ulid/v2"
)

type UserID = ulid.ULID
type NoteID = ulid.ULID
type NotificationID = int32
type UploadID = BinUUID

func NewUserID() UserID {
	return ulid.Make()
}

func ParseUserID(s string) (UserID, error) {
	id, err := ulid.Parse(s)
	return UserID(id), err
}

func NewNoteID() NoteID {
	return ulid.Make()
}

func ParseNoteID(s string) (NoteID, error) {
	id, err := ulid.Parse(s)
	return NoteID(id), err
}

func NewUploadID() UploadID {
	return WrapBinUUID(uuid.New())
}

func ParseUploadID(s string) (UploadID, error) {
	return ParseBinUUID(s)
}

/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

package types

import (
	"strconv"

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

func ParseNotificationID(s string) (NotificationID, error) {
	id, err := strconv.ParseInt(s, 10, 32)
	return NotificationID(id), err
}

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

package apub

import (
	"encoding/json"
	"fmt"
)

type UpdatableObjectType string

const (
	UpdatableObjectTypeNote UpdatableObjectType = "Note"
)

type UpdateActivity struct {
	ID     URI             `json:"id" validate:"required,http_url"`
	Kind   string          `json:"type" validate:"required,eq=Update"`
	Actor  URI             `json:"actor" validate:"required,http_url"`
	To     []string        `json:"to" validate:"dive,http_url"`
	Cc     []string        `json:"cc" validate:"dive,http_url"`
	Object UpdatableObject `json:"object" validate:"required"`
}

func (UpdateActivity) InboxActivity() {}

func (UpdateActivity) IDCheck() error { return nil }

func NewUpdateActivity(object UpdatableObject) UpdateActivity {
	return UpdateActivity{
		ID:     activityIDFromObject("Update", object.ID()),
		Kind:   "Update",
		Actor:  object.Actor(),
		To:     object.To(),
		Cc:     object.Cc(),
		Object: object,
	}
}

type UpdatableObject struct {
	Kind UpdatableObjectType

	NoteObject *NoteObject
}

func (u UpdatableObject) ID() URI {
	switch u.Kind {
	case UpdatableObjectTypeNote:
		return u.NoteObject.ID
	}

	panic("unknown updatable object type")
}

func (u UpdatableObject) Actor() URI {
	switch u.Kind {
	case UpdatableObjectTypeNote:
		return u.NoteObject.AttributedTo
	}

	panic("unknown updatable object type")
}

func (u UpdatableObject) To() []string {
	switch u.Kind {
	case UpdatableObjectTypeNote:
		return u.NoteObject.To
	}

	panic("unknown updatable object type")
}

func (u UpdatableObject) Cc() []string {
	switch u.Kind {
	case UpdatableObjectTypeNote:
		return u.NoteObject.Cc
	}

	panic("unknown updatable object type")
}

func (u UpdatableObject) MarshalJSON() ([]byte, error) {
	switch u.Kind {
	case UpdatableObjectTypeNote:
		return json.Marshal(*u.NoteObject)
	}

	return nil, fmt.Errorf("unknown updatable object type: %s", u.Kind)
}

func (u *UpdatableObject) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Note":
		u.Kind = UpdatableObjectTypeNote
		var n NoteObject
		if err := json.Unmarshal(data, &n); err != nil {
			return fmt.Errorf("error unmarshalling note object: %w", err)
		}
		if err := validate.Struct(n); err != nil {
			return fmt.Errorf("error validating note object: %w", err)
		}
		u.NoteObject = &n
	default:
		return fmt.Errorf("unknown updatable object Type: %s", typ)
	}

	return nil
}

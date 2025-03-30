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

type UndoableActivityType string

const (
	UndoableActivityTypeFollow UndoableActivityType = "Follow"
)

type UndoActivity struct {
	ID     string           `json:"id" validate:"required,http_url"`
	Kind   string           `json:"type" validate:"required,eq=Undo"`
	Actor  URI              `json:"actor" validate:"required,http_url"`
	Object UndoableActivity `json:"object" validate:"required"`
}

func (UndoActivity) InboxActivity() {}

func (a UndoActivity) IDCheck() error {
	switch a.Object.Kind {
	case UndoableActivityTypeFollow:
		f := a.Object.Follow
		if err := f.IDCheck(); err != nil {
			return err
		}
		if f.Actor != a.Actor {
			return fmt.Errorf("actor %s does not match follow activity actor %s", a.Actor, f.Actor)
		}
		return nil
	}

	return fmt.Errorf("unknown undoable activity type: %s", a.Object.Kind)
}

func NewUndoActivity(
	undoer URI,
	undoableActivity UndoableActivity,
) UndoActivity {
	undoID := activityIDFromObject("Undo", undoableActivity.ID())
	return UndoActivity{
		ID:     undoID,
		Kind:   "Undo",
		Actor:  undoer,
		Object: undoableActivity,
	}
}

type UndoableActivity struct {
	Kind UndoableActivityType `validate:"required,oneof=Follow"`

	Follow *FollowActivity
}

func (u UndoableActivity) ID() string {
	switch u.Kind {
	case UndoableActivityTypeFollow:
		return u.Follow.ID
	}

	panic("unknown undoable object type")
}

func (u UndoableActivity) MarshalJSON() ([]byte, error) {
	switch u.Kind {
	case UndoableActivityTypeFollow:
		return json.Marshal(*u.Follow)
	}

	return nil, fmt.Errorf("unknown undoable object type: %s", u.Kind)
}

func (u *UndoableActivity) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Follow":
		u.Kind = UndoableActivityTypeFollow
		var f FollowActivity
		if err := json.Unmarshal(data, &f); err != nil {
			return fmt.Errorf("error unmarshalling follow object: %w", err)
		}
		if err := validate.Struct(f); err != nil {
			return fmt.Errorf("error validating follow object: %w", err)
		}
		u.Follow = &f
	}

	return fmt.Errorf("unknown undoable object type: %s", typ)
}

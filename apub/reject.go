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

type RejectableObjectType string

const (
	RejectableActivityTypeFollow RejectableObjectType = "Follow"
)

type RejectActivity struct {
	ID     string             `json:"id" validate:"required,http_url"`
	Kind   string             `json:"type" validate:"required,eq=Reject"`
	Actor  URI                `json:"actor" validate:"required,http_url"`
	Object RejectableActivity `json:"object" validate:"required"`
}

func (RejectActivity) InboxActivity() {}

func NewRejectActivityWithID(
	rejectID string,
	rejecter URI,
	rejectableObject RejectableActivity,
) RejectActivity {
	return RejectActivity{
		ID:     rejectID,
		Kind:   "Reject",
		Actor:  rejecter,
		Object: rejectableObject,
	}
}

func NewRejectActivity(
	rejecter URI,
	rejectableObject RejectableActivity,
) RejectActivity {
	rejectID := activityIDFromObject("Reject", rejectableObject.ID())
	return NewRejectActivityWithID(
		rejectID,
		rejecter,
		rejectableObject,
	)
}

type RejectableActivity struct {
	Kind RejectableObjectType `validate:"required,oneof=Follow"`

	Follow *FollowActivity
}

func (r RejectableActivity) ID() string {
	switch r.Kind {
	case RejectableActivityTypeFollow:
		return r.Follow.ID
	}

	panic("unknown rejectable object type")
}

func (r RejectableActivity) MarshalJSON() ([]byte, error) {
	switch r.Kind {
	case RejectableActivityTypeFollow:
		return json.Marshal(*r.Follow)
	}

	return nil, fmt.Errorf("unknown rejectable object type: %s", r.Kind)
}

func (r *RejectableActivity) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Follow":
		r.Kind = RejectableActivityTypeFollow
		var f FollowActivity
		if err := json.Unmarshal(data, &f); err != nil {
			return fmt.Errorf("error unmarshalling follow object: %w", err)
		}
		if err := validate.Struct(f); err != nil {
			return fmt.Errorf("error validating follow object: %w", err)
		}
		r.Follow = &f
		return nil
	}

	return fmt.Errorf("unknown rejectable object type: %s", typ)
}

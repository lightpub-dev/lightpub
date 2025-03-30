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
	"bytes"
	"encoding/json"
	"fmt"
	"io"
)

// InboxActivity is a placeholder for activities that can be received in the inbox.
type InboxActivity interface {
	InboxActivity()
}

type InboxActivityData struct {
	Activity InboxActivity
}

func ParseActivityFromBytes(data []byte) (InboxActivity, error) {
	r := bytes.NewReader(data)
	return ParseActivity(r)
}

func ParseActivity(reader io.Reader) (InboxActivity, error) {
	var activity InboxActivityData
	if err := json.NewDecoder(reader).Decode(&activity); err != nil {
		return nil, err
	}
	if err := validate.Struct(activity); err != nil {
		return nil, err
	}
	return activity.Activity, nil
}

func (i InboxActivityData) MarshalJSON() ([]byte, error) {
	return json.Marshal(i.Activity)
}

func (i *InboxActivityData) UnmarshalJSON(data []byte) error {
	_, ty, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch ty {
	case "Accept":
		var activity AcceptActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	case "Announce":
		var activity AnnounceActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	case "Create":
		var activity CreateActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	case "Follow":
		var activity FollowActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	case "Reject":
		var activity RejectActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	case "Undo":
		var activity UndoActivity
		if err := json.Unmarshal(data, &activity); err != nil {
			return err
		}
		i.Activity = activity
	default:
		return fmt.Errorf("unsupported activity type: %s", ty)
	}

	return nil
}

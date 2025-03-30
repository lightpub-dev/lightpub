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
	"strings"

	"github.com/go-playground/validator/v10"
	"github.com/lightpub-dev/lightpub/types"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

func unmarshalToMapAndType(data []byte) (map[string]interface{}, string, error) {
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, "", err
	}

	typ, ok := m["type"].(string)
	if !ok {
		return nil, "", fmt.Errorf("missing Type field")
	}

	return m, typ, nil
}

func inferVisibility(to, cc []string) types.NoteVisibility {
	if containsPublicURL(to) {
		return types.NoteVisibilityPublic
	} else if containsPublicURL(cc) {
		return types.NoteVisibilityUnlisted
	}

	for _, t := range to {
		if strings.HasSuffix(t, followersSuffix) {
			return types.NoteVisibilityFollower
		}
	}

	return types.NoteVisibilityPrivate
}

func activityIDFromObject(activityType string, objectID URI) URI {
	if strings.HasSuffix(objectID, "/") {
		return objectID + activityType
	}
	return objectID + "/" + activityType
}

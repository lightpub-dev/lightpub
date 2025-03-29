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
	"reflect"
	"strings"
)

var (
	appContext = []any{
		"https://www.w3.org/ns/activitystreams",
	}
)

type WithContext[T Signable] struct {
	Data T
}

func (w WithContext[T]) Signer() Actor {
	return w.Data.Signer()
}

func (w WithContext[T]) MarshalJSON() ([]byte, error) {
	fields := make(map[string]any)

	v := reflect.ValueOf(w.Data)
	if v.Kind() == reflect.Ptr || v.Kind() == reflect.Interface {
		v = v.Elem()
	}
	if v.Kind() != reflect.Struct {
		return nil, fmt.Errorf("WithContext: expected struct, got %T", w.Data)
	}

	for i := 0; i < v.NumField(); i++ {
		var key string
		rawKey := v.Type().Field(i).Tag.Get("json")
		// handle ignore
		if rawKey == "-" {
			continue
		}
		// handle omitempty
		if strings.HasSuffix(rawKey, ",omitempty") {
			key = strings.TrimSuffix(rawKey, ",omitempty")
			if v.Field(i).IsZero() {
				continue
			}
		} else {
			key = rawKey
		}
		if key == "" {
			key = v.Type().Field(i).Name
		}

		fields[key] = v.Field(i).Interface()
	}

	fields["@context"] = appContext

	return json.Marshal(fields)
}

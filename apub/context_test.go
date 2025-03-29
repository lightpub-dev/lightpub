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
	"testing"

	"github.com/google/go-cmp/cmp"
)

type TestStruct struct {
	Name  string `json:"name"`
	Value int    `json:"value,omitempty"`
}

func (TestStruct) Signer() Actor {
	return nil
}

type invalidString string

func (invalidString) Signer() Actor {
	return nil
}

func TestWithContext_MarshalJSON(t *testing.T) {
	tests := []struct {
		name    string
		data    Signable
		want    map[string]any
		wantErr bool
	}{
		{
			name: "simple struct",
			data: TestStruct{Name: "test", Value: 42},
			want: map[string]any{
				"name":     "test",
				"value":    float64(42),
				"@context": appContext,
			},
			wantErr: false,
		},
		{
			name: "omitempty field",
			data: TestStruct{Name: "test"},
			want: map[string]any{
				"name":     "test",
				"@context": appContext,
			},
			wantErr: false,
		},
		{
			name:    "non-struct data",
			data:    invalidString("invalid"),
			want:    nil,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			w := WithContext[Signable]{Data: tt.data}
			got, err := w.MarshalJSON()
			if (err != nil) != tt.wantErr {
				t.Errorf("WithContext.MarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				var gotMap map[string]any
				if err := json.Unmarshal(got, &gotMap); err != nil {
					t.Errorf("json.Unmarshal() error = %v", err)
					return
				}
				if diff := cmp.Diff(gotMap, tt.want); diff != "" {
					t.Errorf("WithContext.MarshalJSON() mismatch (-got +want):\n%s", diff)
				}
			}
		})
	}
}

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

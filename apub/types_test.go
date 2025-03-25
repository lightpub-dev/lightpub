package apub_test

import (
	"testing"

	"github.com/lightpub-dev/lightpub/apub"
)

func TestObjectID_UnmarshalJSON(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		want    string
		wantErr bool
	}{
		{
			name:    "valid string ID",
			input:   `"12345"`,
			want:    "12345",
			wantErr: false,
		},
		{
			name:    "valid object ID",
			input:   `{"id": "67890"}`,
			want:    "67890",
			wantErr: false,
		},
		{
			name:    "empty object ID",
			input:   `{"id": ""}`,
			wantErr: true,
		},
		{
			name:    "invalid JSON",
			input:   `{"id": 12345}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var o apub.ObjectID
			err := o.UnmarshalJSON([]byte(tt.input))
			if (err != nil) != tt.wantErr {
				t.Errorf("UnmarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if o.ID != tt.want {
				t.Errorf("UnmarshalJSON() got = %v, want %v", o.ID, tt.want)
			}
		})
	}
}

func TestObjectID_MarshalJSON(t *testing.T) {
	tests := []struct {
		name    string
		input   apub.ObjectID
		want    string
		wantErr bool
	}{
		{
			name:    "valid ID",
			input:   apub.ObjectID{ID: "12345"},
			want:    `"12345"`,
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.input.MarshalJSON()
			if (err != nil) != tt.wantErr {
				t.Errorf("MarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if string(got) != tt.want {
				t.Errorf("MarshalJSON() got = %v, want %v", string(got), tt.want)
			}
		})
	}
}

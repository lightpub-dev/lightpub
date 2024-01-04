package testutils

import (
	"bytes"
	"encoding/json"
	"testing"

	"github.com/go-playground/validator/v10"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

func SchemaCheck(t *testing.T, dst interface{}, response *bytes.Buffer) {
	if err := json.Unmarshal(response.Bytes(), dst); err != nil {
		t.Fatalf("failed to unmarshal response: %s\nBody:\n%v", err, string(response.String()))
		return
	}

	if err := validate.Struct(dst); err != nil {
		t.Fatalf("failed to validate response: %s", err)
		return
	}
}

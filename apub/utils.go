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

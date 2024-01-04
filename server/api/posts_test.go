package api_test

import (
	"testing"

	"github.com/lightpub-dev/lightpub/testutils"
	"github.com/stretchr/testify/assert"
)

type postResponse struct {
	ID string `json:"id" validate:"required,uuid"`
}

func TestNormalPost(t *testing.T) {
	dbInit(t, testutils.DefaultDBConnection())

	login := createAdminUser(t)

	r := setupRequestAuth(t, "POST", "/post", login.Token, map[string]interface{}{
		"content": "Hello @ lightpub",
		"privacy": "public",
	})

	assert.Equal(t, 201, r.Rec.Code)
	var result postResponse
	testutils.SchemaCheck(t, &result, r.Rec.Body)
}

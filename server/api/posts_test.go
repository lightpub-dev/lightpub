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

func TestReplyPost(t *testing.T) {
	dbInit(t, testutils.DefaultDBConnection())

	login := createAdminUser(t)
	login2 := createAdminUser2(t)

	publicPost := setupRequestAuth(t, "POST", "/post", login.Token, map[string]interface{}{
		"content": "Hello @ lightpub",
		"privacy": "public",
	})

	assert.Equal(t, 201, publicPost.Rec.Code)
	var result postResponse
	testutils.SchemaCheck(t, &result, publicPost.Rec.Body)

	privatePost := setupRequestAuth(t, "POST", "/post", login.Token, map[string]interface{}{
		"content": "very private info",
		"privacy": "private",
	})

	assert.Equal(t, 201, privatePost.Rec.Code)
	var resultP postResponse
	testutils.SchemaCheck(t, &resultP, privatePost.Rec.Body)

	t.Run("Reply to public post", func(t *testing.T) {
		replyPost := setupRequestAuth(t, "POST", "/post/"+result.ID+"/reply", login2.Token, map[string]interface{}{
			"content": "this is reply",
			"privacy": "public",
		})

		var result postResponse
		assert.Equal(t, 201, replyPost.Rec.Code)
		testutils.SchemaCheck(t, &result, replyPost.Rec.Body)
	})

	t.Run("Reply to private post by unprivileged user", func(t *testing.T) {
		failedReplyPost := setupRequestAuth(t, "POST", "/post/"+resultP.ID+"/reply", login2.Token, map[string]interface{}{
			"content": "this is reply",
			"privacy": "public",
		})

		assert.Equal(t, 404, failedReplyPost.Rec.Code)
	})

	t.Run("Reply to private post by original poster", func(t *testing.T) {
		failedReplyPost := setupRequestAuth(t, "POST", "/post/"+resultP.ID+"/reply", login.Token, map[string]interface{}{
			"content": "this is reply by original poster",
			"privacy": "public",
		})

		assert.Equal(t, 201, failedReplyPost.Rec.Code)
	})
}

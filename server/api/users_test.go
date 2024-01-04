package api_test

import (
	"testing"

	"github.com/lightpub-dev/lightpub/testutils"
	"github.com/stretchr/testify/assert"
)

func TestUserRegistration(t *testing.T) {
	dbInit(t, testutils.DefaultDBConnection())

	r := setupRequest(t, "POST", "/register", map[string]interface{}{
		"username": "admin",
		"password": "1234abcd",
		"nickname": "admin dayo",
	})

	assert.Equal(t, 201, r.Rec.Code)
}

type loginInfo struct {
	Username string
	Token    string
}

type createUserInfo struct {
	Username string `json:"username" validate:"required"`
	Password string `json:"password" validate:"required"`
	Nickname string `json:"nickname" validate:"required"`
}

func createUser(t *testing.T, create createUserInfo) loginInfo {
	t.Helper()

	r := setupRequest(t, "POST", "/register", map[string]interface{}{
		"username": create.Username,
		"password": create.Password,
		"nickname": create.Nickname,
	})

	assert.Equal(t, 201, r.Rec.Code)

	r = setupRequest(t, "POST", "/login", map[string]interface{}{
		"username": create.Username,
		"password": create.Password,
	})

	assert.Equal(t, 200, r.Rec.Code)
	// must have token
	var result loginResult
	testutils.SchemaCheck(t, &result, r.Rec.Body)
	return loginInfo{
		Username: create.Username,
		Token:    result.Token,
	}
}

func createAdminUser(t *testing.T) loginInfo {
	return createUser(t, createUserInfo{
		Username: "admin",
		Password: "1234abcd",
		Nickname: "admin dayo",
	})
}

func createAdminUser2(t *testing.T) loginInfo {
	return createUser(t, createUserInfo{
		Username: "useruser",
		Password: "1234user",
		Nickname: "user dayo",
	})
}

type loginResult struct {
	Token string `json:"token" validate:"required,uuid"`
}

func TestUserLogin(t *testing.T) {
	dbInit(t, testutils.DefaultDBConnection())

	r := setupRequest(t, "POST", "/register", map[string]interface{}{
		"username": "admin",
		"password": "1234abcd",
		"nickname": "admin dayo",
	})

	assert.Equal(t, 201, r.Rec.Code)

	t.Run("login correctly", func(t *testing.T) {
		r := setupRequest(t, "POST", "/login", map[string]interface{}{
			"username": "admin",
			"password": "1234abcd",
		})

		assert.Equal(t, 200, r.Rec.Code)
		// must have token
		var result loginResult
		testutils.SchemaCheck(t, &result, r.Rec.Body)
	})

	t.Run("login fail", func(t *testing.T) {
		r := setupRequest(t, "POST", "/login", map[string]interface{}{
			"username": "admin",
			"password": "1234abc",
		})

		assert.Equal(t, 401, r.Rec.Code)
	})
}

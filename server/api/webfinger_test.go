package api_test

import (
	"net/http"
	"testing"

	"github.com/lightpub-dev/lightpub/testutils"
	"github.com/stretchr/testify/assert"
)

func TestWebfinger(t *testing.T) {
	dbInit(t, testutils.DefaultDBConnection())

	createAdminUser(t)

	t.Run("Username only", func(t *testing.T) {
		r := setupRequest(t, "GET", "/.well-known/webfinger?resource=acct:admin", nil)

		assert.Equal(t, 200, r.Rec.Code)
	})

	t.Run("Username and host", func(t *testing.T) {
		r := setupRequest(t, "GET", "/.well-known/webfinger?resource=acct:admin@localhost:1323", nil)

		assert.Equal(t, 200, r.Rec.Code)
	})

	t.Run("Invalid resource", func(t *testing.T) {
		r := setupRequest(t, "GET", "/.well-known/webfinger?resource=acc:admin@localhost:1323", nil)

		assert.Equal(t, http.StatusUnprocessableEntity, r.Rec.Code)
	})

	t.Run("Invalid host", func(t *testing.T) {
		r := setupRequest(t, "GET", "/.well-known/webfinger?resource=acct:admin@example.com", nil)

		assert.Equal(t, http.StatusUnprocessableEntity, r.Rec.Code)
	})
}

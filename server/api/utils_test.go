package api_test

import (
	"fmt"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/api"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/testutils"
)

func dbInit(t *testing.T, conn db.DBConnectionInfo) {
	t.Helper()
	if err := db.MigrateToLatest(conn, "../migrations", false); err != nil {
		t.Fatal(err)
	}

	if err := testutils.TruncateAll(conn); err != nil {
		t.Fatal(err)
	}

	if err := db.MigrateToLatest(conn, "../migrations", false); err != nil {
		t.Fatal(err)
	}
}

type request struct {
	Echo    *echo.Echo
	Handler *api.Handler
	Rec     *httptest.ResponseRecorder
}

func setupRequest(t *testing.T, method string, path string, body interface{}) request {
	t.Helper()
	e, h := testutils.DefaultEcho()
	req := httptest.NewRequest(method, path, testutils.NewJSONBody(body))
	req.Header.Add("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	return request{
		Echo:    e,
		Handler: h,
		Rec:     rec,
	}
}

func setupRequestAuth(t *testing.T, method string, path string, token string, body interface{}) request {
	t.Helper()
	e, h := testutils.DefaultEcho()
	req := httptest.NewRequest(method, path, testutils.NewJSONBody(body))
	req.Header.Add("Content-Type", "application/json")
	req.Header.Add("Authorization", fmt.Sprintf("Bearer %s", token))
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)

	return request{
		Echo:    e,
		Handler: h,
		Rec:     rec,
	}
}

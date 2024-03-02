package db_test

import (
	"testing"

	"github.com/google/uuid"
	"github.com/lightpub-dev/lightpub/db"
)

func TestUuidToString(t *testing.T) {
	u := uuid.MustParse("a5911421-7a42-4a60-9b53-5c51e75073b5")
	hex := db.UuidToString(u)
	if hex != "a59114217a424a609b535c51e75073b5" {
		t.Errorf("Expected: %v, Got: %v", "a59114217a424a609b535c51e75073b5", hex)
	}
}

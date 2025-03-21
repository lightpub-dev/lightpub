package service

import (
	"net/url"
	"testing"

	"github.com/lightpub-dev/lightpub/types"
)

func TestExtractUserIDFromLocalURL(t *testing.T) {
	u, err := url.Parse("https://example.com/user/01ARZ3NDEKTSV4RRFFQ69G5FAV")
	if err != nil {
		panic(err)
	}
	userID, err := types.ParseUserID("01ARZ3NDEKTSV4RRFFQ69G5FAV")
	if err != nil {
		panic(err)
	}

	parsedUserID, ok := extractUserIDFromLocalURL(u, "example.com")
	if !ok {
		t.Fatalf("expected to extract user ID from URL, but failed")
	}
	if parsedUserID != userID {
		t.Fatalf("expected user ID %v, but got %v", userID, parsedUserID)
	}
}

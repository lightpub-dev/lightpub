package service

import (
	"net/url"
	"strings"

	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) extractUserIDFromLocalURL(url *url.URL) (types.UserID, bool) {
	return extractUserIDFromLocalURL(url, s.MyDomain())
}

func extractUserIDFromLocalURL(url *url.URL, myDomain string) (types.UserID, bool) {
	if url.Host != myDomain {
		return types.UserID{}, false
	}

	trimmedPath := strings.TrimPrefix(url.Path, "/")
	// expects "user/<ULID>"
	if !strings.HasPrefix(trimmedPath, "user/") {
		return types.UserID{}, false
	}

	userIDStr := strings.TrimPrefix(trimmedPath, "user/")
	userID, err := types.ParseUserID(userIDStr)
	if err != nil {
		return types.UserID{}, false
	}
	return userID, true
}

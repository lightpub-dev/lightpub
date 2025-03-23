/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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

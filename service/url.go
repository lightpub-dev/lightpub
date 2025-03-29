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
	"strconv"
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

func (s *State) followersURLForLocalUser(userID types.UserID) string {
	return s.BaseURL().JoinPath("user", userID.String(), "followers").String()
}

func (s *State) urlForLocalUser(userID types.UserID) string {
	return s.BaseURL().JoinPath("user", userID.String()).String()
}

func (s *State) viewURLForLocalUser(userID types.UserID) string {
	return s.BaseURL().JoinPath("client", "user", userID.String()).String()
}

func (s *State) keyIDForLocalUser(userID types.UserID) string {
	keyID := s.BaseURL().JoinPath("user", userID.String())
	keyID.Fragment = "main-key"
	return keyID.String()
}

type collectionURL struct {
	Followers string
	Following string
	Likes     string
}

func (s *State) collectionURLsForLocalUser(userID types.UserID) collectionURL {
	return collectionURL{
		Followers: s.followersURLForLocalUser(userID),
		Following: s.BaseURL().JoinPath("user", userID.String(), "following").String(),
		Likes:     s.BaseURL().JoinPath("user", userID.String(), "likes").String(),
	}
}

func (s *State) inboxForLocalUser(userID types.UserID) string {
	return s.BaseURL().JoinPath("user", userID.String(), "inbox").String()
}

func (s *State) outboxForLocalUser(userID types.UserID) string {
	return s.BaseURL().JoinPath("user", userID.String(), "outbox").String()
}

func (s *State) sharedInboxForLocalUser() string {
	return s.BaseURL().JoinPath("inbox").String()
}

func (s *State) urlForLocalNote(noteID types.NoteID) string {
	return s.BaseURL().JoinPath("note", noteID.String()).String()
}

func (s *State) viewURLForLocalNote(noteID types.NoteID) string {
	return s.BaseURL().JoinPath("client", "note", noteID.String()).String()
}

func (s *State) viewURLForHashtag(hashtag string) string {
	u := s.BaseURL().JoinPath("client", "timeline")
	u.Query().Add("tag", hashtag)
	return u.String()
}

func (s *State) urlForLocalUpload(uploadID types.UploadID) string {
	return s.BaseURL().JoinPath("upload", uploadID.String()).String()
}

func (s *State) urlForLocalFollowReject(followID int) string {
	return s.BaseURL().JoinPath("follow", strconv.Itoa(followID), "reject").String()
}

func (s *State) urlForLocalFollowAccept(followID int) string {
	return s.BaseURL().JoinPath("follow", strconv.Itoa(followID), "accept").String()
}

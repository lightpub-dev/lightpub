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

package apub

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"time"

	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/types"
)

var (
	ErrInvalidUserID = failure.NewError(http.StatusBadRequest, "invalid URL in Remote actor object")
)

type User struct {
	ID                string `json:"id" validate:"required,http_url"`
	Kind              string `json:"type" validate:"required,oneof=Person Application Service"`
	Name              string `json:"name"`                                  // Nickname, optional
	PreferredUsername string `json:"preferredUsername" validate:"required"` // Username, required
	Summary           string `json:"summary,omitempty"`                     // Bio, optional

	Inbox     string       `json:"inbox,omitempty" validate:"http_url"`     // Optional
	Outbox    string       `json:"outbox,omitempty" validate:"http_url"`    // Optional
	Following string       `json:"following,omitempty" validate:"http_url"` // Optional
	Followers string       `json:"followers,omitempty" validate:"http_url"` // Optional
	Endpoints UserEndpoint `json:"endpoints,omitempty"`                     // Optional
	URL       string       `json:"url,omitempty" validate:"http_url"`       // Optional

	Published                 *time.Time `json:"published,omitempty"`                 // Registered timestamp, Optional
	ManuallyApprovesFollowers bool       `json:"manuallyApprovesFollowers,omitempty"` // Optional
	Icon                      *UserIcon  `json:"icon,omitempty"`                      // Optional

	PublicKey UserPublicKey `json:"publicKey" validate:"required"` // PublicKey, Required
}

type UserPublicKey struct {
	ID           string `json:"id" validate:"required,http_url"`
	Owner        string `json:"owner" validate:"required,http_url"`
	PublicKeyPem string `json:"publicKeyPem" validate:"required"` // PEM encoded public key
}

type UserEndpoint struct {
	SharedInbox string `json:"sharedInbox,omitempty" validate:"http_url"` // Optional
}

type UserIcon struct {
	Kind string `json:"type" validate:"required,oneof=Image"`
	URL  string `json:"url" validate:"required,http_url"`
}

// NewUser creates a new User object from an ApubUser object.
// ApubUser must be a local user.
func NewUser(
	user types.ApubUser,
) (*User, error) {
	// local check
	if user.Basic.IsRemote() {
		return nil, fmt.Errorf("user is not local")
	}

	kind := "Person" // TODO: handle bot

	var icon *UserIcon
	// TODO: set icon

	publicKey, err := encodePublicKey(user.Apub.PublicKey_)
	if err != nil {
		return nil, fmt.Errorf("failed to create ApubUser: %w", err)
	}

	return &User{
		ID:                user.Apub.URL,
		Kind:              kind,
		Name:              user.Basic.Nickname,
		PreferredUsername: user.Basic.Username,
		Summary:           user.Basic.Bio,

		Inbox:     user.Apub.Inbox,
		Outbox:    user.Apub.Outbox,
		Following: user.Apub.Following,
		Followers: user.Apub.Followers,
		Endpoints: UserEndpoint{
			SharedInbox: user.Apub.SharedInbox,
		},
		URL: user.Apub.ViewURL,

		Published:                 nil,
		ManuallyApprovesFollowers: user.Apub.ManuallyApprovesFollowers,
		Icon:                      icon,

		PublicKey: UserPublicKey{
			ID:           user.Apub.KeyID_,
			Owner:        user.Apub.URL,
			PublicKeyPem: publicKey,
		},
	}, nil
}

func (s *Requester) FetchRemoteUserBySpecifier(ctx context.Context, specifier *types.UserSpecifier) (*User, error) {
	switch specifier.Kind {
	case types.UserSpecifierID:
		return nil, fmt.Errorf("cannot fetch remote user by ID")
	case types.UserSpecifierUsername:
		// use webfinger to get URL
		userURL, err := s.fetchUserURLByWebfinger(ctx, specifier.Username.Username, specifier.Username.Domain)
		if err != nil {
			slog.DebugContext(ctx, "failed to fetch user URL by webfinger", "username", specifier.Username.Username, "domain", specifier.Username.Domain, "error", err)
			return nil, failure.NewErrorWithCause(http.StatusNotFound, "webfinger error", err)
		}
		return s.fetchRemoteUser(ctx, userURL)
	case types.UserSpecifierURL:
		// fetch
		return s.fetchRemoteUser(ctx, specifier.URL)
	}

	panic("unreachable")
}

func (s *Requester) fetchRemoteUser(ctx context.Context, url *url.URL) (*User, error) {
	// Create a new HTTP request
	req, err := http.NewRequestWithContext(ctx, "GET", url.String(), nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Set Accept header for ActivityPub content types
	req.Header.Set("Accept", "application/activity+json, application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")

	// Perform the HTTP request
	resp, err := s.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	// Check if the response status is OK
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("request returned status: %d", resp.StatusCode)
	}

	// Parse the JSON response into the User struct
	var user User
	if err := json.NewDecoder(resp.Body).Decode(&user); err != nil {
		return nil, fmt.Errorf("failed to decode actor JSON: %w", err)
	}
	if err := validate.Struct(user); err != nil {
		return nil, fmt.Errorf("failed to validate actor JSON: %w", err)
	}
	if err := s.userSecurityCheck(url.Host, &user); err != nil {
		return nil, fmt.Errorf("failed to validate actor JSON: %w", err)
	}

	return &user, nil
}

func (s *Requester) userSecurityCheck(expectedOrigin string, user *User) error {
	userID, err := url.Parse(user.ID)
	if err != nil {
		return ErrInvalidUserID
	}
	if userID.Host != expectedOrigin {
		return failure.NewError(http.StatusBadRequest, "invalid URL host in actor.id")
	}

	if user.PublicKey.Owner != user.ID {
		return failure.NewError(http.StatusBadRequest, "key owner does not match actor.id")
	}
	pubKeyID, err := url.Parse(user.PublicKey.ID)
	if err != nil {
		return failure.NewError(http.StatusBadRequest, "invalid URL in actor.publicKey.id")
	}
	if pubKeyID.Host != expectedOrigin {
		return failure.NewError(http.StatusBadRequest, "invalid URL host in actor.publicKey.id")
	}

	return nil
}

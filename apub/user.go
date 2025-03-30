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
	"fmt"
	"net/http"
	"net/url"
	"time"

	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/types"
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

func (s *Requester) fetchRemoteUserAndStore(ctx context.Context, specifier *types.UserSpecifier) (*types.ApubUser, error) {
	switch specifier.Kind {
	case types.UserSpecifierID:
		return nil, fmt.Errorf("cannot fetch remote user by ID")
	case types.UserSpecifierUsername:
		// use webfinger to get URL
		userURL, err := s.fetchUserURLByWebfinger(ctx, specifier.Username.Username, specifier.Username.Domain)
		if err != nil {
			return nil, failure.NewErrorWithCause(http.StatusNotFound, "failed to fetch remote user", err)
		}
		return s.fetchRemoteUser(ctx, userURL)
	case types.UserSpecifierURL:
		// fetch
		return s.fetchRemoteUser(ctx, specifier.URL)
	}

	panic("unreachable")
}

func (s *Requester) fetchRemoteUser(ctx context.Context, url *url.URL) (*types.ApubUser, error) {
	return nil, nil
}

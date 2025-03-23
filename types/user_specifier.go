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

package types

import (
	"fmt"
	"net/url"
	"strings"
)

// UserSpecifierType represents the type of user identifier
type UserSpecifierType int

const (
	UserSpecifierID UserSpecifierType = iota
	UserSpecifierUsername
	UserSpecifierURL
)

// UsernameAndDomain holds username and optional domain
type UsernameAndDomain struct {
	Username string
	Domain   string // Empty string means local domain
}

// UserSpecifier represents different ways to specify a user
type UserSpecifier struct {
	Kind UserSpecifierType

	// Only one of these fields is set based on Kind
	ID       UserID
	Username UsernameAndDomain
	URL      *url.URL
}

// FromStr parses a string into a UserSpecifier
func ParseUserSpecifier(spec string, myDomain string) (*UserSpecifier, bool) {
	if !strings.HasPrefix(spec, "@") {
		id, err := ParseUserID(spec)
		if err != nil {
			return nil, false
		}
		return &UserSpecifier{
			Kind: UserSpecifierID,
			ID:   id,
		}, true
	}

	atSplit := strings.Split(spec, "@")

	switch len(atSplit) {
	case 2:
		return &UserSpecifier{
			Kind: UserSpecifierUsername,
			Username: UsernameAndDomain{
				Username: atSplit[1],
				Domain:   "", // Local domain
			},
		}, true
	case 3:
		domain := atSplit[2]
		if domain == myDomain {
			domain = "" // Local domain
		}
		return &UserSpecifier{
			Kind: UserSpecifierUsername,
			Username: UsernameAndDomain{
				Username: atSplit[1],
				Domain:   domain,
			},
		}, true
	default:
		return nil, false
	}
}

// NewLocalUsername creates a UserSpecifier for a local username
func NewLocalUsername(username string) *UserSpecifier {
	return &UserSpecifier{
		Kind: UserSpecifierUsername,
		Username: UsernameAndDomain{
			Username: username,
			Domain:   "",
		},
	}
}

// NewUsernameAndDomain creates a UserSpecifier with username and domain
func NewUsernameAndDomain(username string, domain string) *UserSpecifier {
	if domain == "" {
		return NewLocalUsername(username)
	}
	return &UserSpecifier{
		Kind: UserSpecifierUsername,
		Username: UsernameAndDomain{
			Username: username,
			Domain:   domain,
		},
	}
}

// NewURL creates a UserSpecifier from a URL
func NewURL(url *url.URL) *UserSpecifier {
	return &UserSpecifier{
		Kind: UserSpecifierURL,
		URL:  url,
	}
}

// OmitMyDomain converts a domain to local if it matches myDomain
// Receiver is not modified. A new UserSpecifier is returned.
func (u *UserSpecifier) OmitMyDomain(myDomain string) *UserSpecifier {
	if u.Kind == UserSpecifierUsername && u.Username.Domain == myDomain {
		return &UserSpecifier{
			Kind: UserSpecifierUsername,
			Username: UsernameAndDomain{
				Username: u.Username.Username,
				Domain:   "",
			},
		}
	}
	return u
}

// String returns a string representation of the UserSpecifier
func (u UserSpecifier) String() string {
	switch u.Kind {
	case UserSpecifierID:
		return u.ID.String()
	case UserSpecifierURL:
		return u.URL.String()
	case UserSpecifierUsername:
		if u.Username.Domain != "" {
			return fmt.Sprintf("@%s@%s", u.Username.Username, u.Username.Domain)
		}
		return fmt.Sprintf("@%s", u.Username.Username)
	default:
		return ""
	}
}

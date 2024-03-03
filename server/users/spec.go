package users

import (
	"errors"
	"net/url"
	"strings"

	"github.com/lightpub-dev/lightpub/db"
)

type SpecifierType int

var (
	ErrInvalidUserSpec = errors.New("invalid user specifier")
)

const (
	SpecifierTypeID SpecifierType = iota
	SpecifierTypeUsernameAndHost
	SpecifierURI
)

type UsernameAndHost struct {
	Username string
	Host     string // empty if local user
}

func NewUsernameAndHost(username, host string) UsernameAndHost {
	return UsernameAndHost{Username: username, Host: host}
}

type Specifier struct {
	Type            SpecifierType
	ID              db.UUID // uuid in the local database
	UsernameAndHost UsernameAndHost
	URI             *url.URL
}

func NewSpecifierFromID(id db.UUID) Specifier {
	return Specifier{Type: SpecifierTypeID, ID: id}
}

func NewSpecifierFromUsernameAndHost(username, host string) Specifier {
	return Specifier{Type: SpecifierTypeUsernameAndHost, UsernameAndHost: NewUsernameAndHost(username, host)}
}

func NewSpecifierFromURI(uri *url.URL) Specifier {
	return Specifier{Type: SpecifierURI, URI: uri}
}

func ParseUserSpec(s string) (Specifier, error) {
	// if not s contains no @, it is an ID
	if !strings.Contains(s, "@") {
		// try to parse a UUID
		var uuid *db.UUID
		if err := db.ParseTo(uuid, s); err != nil {
			return Specifier{}, ErrInvalidUserSpec
		}
		if uuid != nil {
			return NewSpecifierFromID(*uuid), nil
		}
		return Specifier{}, ErrInvalidUserSpec
	}

	// if not, must start with @
	if !strings.HasPrefix(s, "@") {
		return Specifier{}, ErrInvalidUserSpec
	}

	rest := s[1:]
	// split it by @ to separate username and host
	parts := strings.Split(rest, "@")
	switch len(parts) {
	case 1:
		return NewSpecifierFromUsernameAndHost(parts[0], ""), nil
	case 2:
		return NewSpecifierFromUsernameAndHost(parts[0], parts[1]), nil
	default:
		return Specifier{}, ErrInvalidUserSpec
	}
}

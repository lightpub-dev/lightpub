package users

import (
	"net/url"

	"github.com/lightpub-dev/lightpub/db"
)

type SpecifierType int

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

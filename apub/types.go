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
	"crypto"
	"encoding/json"
	"fmt"
)

const (
	PublicURL = "https://www.w3.org/ns/activitystreams#Public"

	ApubActivityJsonType = "application/activity+json"
	ApubLdJsonType       = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\""
)

var (
	publicURLs = []string{
		PublicURL,
		"Public",
		"as:Public",
	}
	followersSuffix = "/followers"
)

type URI = string

func containsPublicURL(urls []string) bool {
	for _, url := range urls {
		for _, publicURL := range publicURLs {
			if url == publicURL {
				return true
			}
		}
	}
	return false
}

type ObjectID struct {
	ID URI `validate:"required,http_url"`
}

func NewObjectID(id URI) ObjectID {
	return ObjectID{ID: id}
}

func (o *ObjectID) UnmarshalJSON(data []byte) error {
	var id URI
	err := json.Unmarshal(data, &id)
	if err == nil {
		o.ID = id
		return nil
	}

	var obj struct {
		ID string `json:"id"`
	}
	err = json.Unmarshal(data, &obj)
	if err != nil {
		return err
	}
	if obj.ID == "" {
		return fmt.Errorf("empty ID")
	}

	o.ID = obj.ID
	return nil
}

func (o ObjectID) MarshalJSON() ([]byte, error) {
	return json.Marshal(o.ID)
}

type Object interface {
	// ID returns the ID of the object.
	ID() string
}

type Actor interface {
	Object
	// PublicKey returns the public key of the actor.
	PublicKey() crypto.PublicKey
	// PrivateKey returns the private key of the actor.
	// If the actor is a remote actor, this should return an empty string.
	PrivateKey() crypto.PrivateKey
	// Key ID returns the key ID of the actor.
	KeyID() string
}

// Signable is an interface for objects that can be signed.
type Signable interface {
	// Signer returns the actor of the object.
	Signer() Actor
}

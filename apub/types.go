package apub

import (
	"crypto"
	"encoding/json"
	"fmt"
)

const (
	PublicURL = "https://www.w3.org/ns/activitystreams#Public"
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

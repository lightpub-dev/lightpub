package pub

import (
	"crypto"

	"github.com/lightpub-dev/lightpub/db"
)

type PrivateKey struct {
	Key   crypto.PrivateKey
	KeyID string
}

func NewPrivateKey(key crypto.PrivateKey, keyID string) PrivateKey {
	return PrivateKey{Key: key, KeyID: keyID}
}

type KeyResolveService interface {
	ResolvePublicKey(keyID string) (crypto.PublicKey, error)
	ResolvePrivateKey(user *db.User) (PrivateKey, error)
}

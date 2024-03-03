package pub

import "github.com/lightpub-dev/lightpub/db"

type KeyResolveService interface {
	ResolvePublicKey(keyID string) (string, error)
	ResolvePrivateKey(user *db.User) (string, error)
}

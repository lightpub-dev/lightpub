package api

import (
	"crypto"
	"crypto/x509"
	"encoding/pem"
	"errors"
	"log"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
)

type DBKeyResolveService struct {
	conn     db.DBConn
	idGetter pub.IDGetterService
}

func ProvideDBKeyResolveService(conn db.DBConn, idGetter pub.IDGetterService) *DBKeyResolveService {
	return &DBKeyResolveService{conn: conn, idGetter: idGetter}
}

func (s *DBKeyResolveService) ResolvePublicKey(keyID string) (crypto.PublicKey, error) {
	log.Fatalf("not implemented")
	return nil, nil
}

func (s *DBKeyResolveService) ResolvePrivateKey(user *db.User) (pub.PrivateKey, error) {
	if !user.PrivateKey.Valid {
		return pub.PrivateKey{}, errors.New("private key not found")
	}

	keyID, err := s.idGetter.GetUserID(user, "publicKey")
	if err != nil {
		return pub.PrivateKey{}, err
	}

	block, _ := pem.Decode([]byte(user.PrivateKey.String))
	if block == nil {
		return pub.PrivateKey{}, errors.New("failed to decode private key stored in the db")
	}

	privateKey, err := x509.ParsePKCS8PrivateKey(block.Bytes)
	if err != nil {
		return pub.PrivateKey{}, err
	}

	privateKey, ok := privateKey.(crypto.PrivateKey)
	if !ok {
		return pub.PrivateKey{}, errors.New("failed to parse private key")
	}

	return pub.NewPrivateKey(privateKey, keyID.String()), nil
}

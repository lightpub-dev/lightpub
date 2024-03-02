package users

import (
	"database/sql"
	"log"
	"os"
	"os/exec"
	"path"
	"strconv"

	"github.com/lightpub-dev/lightpub/db"
)

type UserKeyService interface {
	GenerateKeyForUser(userID db.UUID) error
}

type DBUserKeyService struct {
	conn db.DBConn
}

func ProvideDBUserKeyService(conn db.DBConn) *DBUserKeyService {
	return &DBUserKeyService{conn}
}

type keyPair struct {
	PrivateKey string
	PublicKey  string
}

func (s *DBUserKeyService) GenerateKeyForUser(userID db.UUID) error {
	keyPair, err := generateKeyPair()
	if err != nil {
		log.Printf("error generating key pair for user %s: %s", userID, err)
		return err
	}

	if err := s.conn.DB.Model(&db.User{ID: userID}).Updates(&db.User{
		PrivateKey: sql.NullString{String: keyPair.PrivateKey, Valid: true},
		PublicKey:  sql.NullString{String: keyPair.PublicKey, Valid: true},
	}).Error; err != nil {
		log.Printf("error updating key pair for user %s: %s", userID, err)
	}

	log.Printf("generated key pair for user %s", userID)
	return nil
}

func generateKeyPair() (keyPair, error) {
	// create a temporary directory
	tmpDir, err := os.MkdirTemp("", "lightpub-keygen")
	if err != nil {
		return keyPair{}, err
	}
	// clean up directory after finish
	defer os.RemoveAll(tmpDir)
	// chmod 700
	if err := os.Chmod(tmpDir, 0700); err != nil {
		return keyPair{}, err
	}

	// generate key pair using OpenSSL
	keyPairFile := path.Join(tmpDir, "keypair.pem")
	bits := 2048
	cmd := exec.Command("openssl", "genrsa", "-out", keyPairFile, strconv.Itoa(bits))
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}

	// extract public key
	publicKeyFile := path.Join(tmpDir, "public.pem")
	cmd = exec.Command("openssl", "rsa", "-in", keyPairFile, "-pubout", "-out", publicKeyFile)
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}
	// read public key
	publicKey, err := os.ReadFile(publicKeyFile)
	if err != nil {
		return keyPair{}, err
	}

	// export private key as pkcs8
	privateKeyFile := path.Join(tmpDir, "private.pem")
	cmd = exec.Command("openssl", "pkcs8", "-topk8", "-inform", "PEM", "-outform", "PEM", "-nocrypt", "-in", keyPairFile, "-out", privateKeyFile)
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}
	// read private key
	privateKey, err := os.ReadFile(privateKeyFile)
	if err != nil {
		return keyPair{}, err
	}

	publicKeyString := string(publicKey)
	privateKeyString := string(privateKey)

	return keyPair{
		PrivateKey: privateKeyString,
		PublicKey:  publicKeyString,
	}, nil
}

package auth

import (
	"log"
	"os"
)

type State struct {
	jwtPublicKey  []byte
	jwtPrivateKey []byte
}

type Config struct {
	JWTPublicKeyPath  string `yaml:"jwt_public_key_path"`
	JWTPrivateKeyPath string `yaml:"jwt_private_key_path"`
}

func NewStateFromConfig(config Config) *State {
	jwtPublicKey, err := readJWTPublicKey(config.JWTPublicKeyPath)
	if err != nil {
		log.Fatalf("failed to read JWT public key: %v", err)
	}

	jwtPrivateKey, err := readJWTPrivateKey(config.JWTPrivateKeyPath)
	if err != nil {
		log.Fatalf("failed to read JWT private key: %v", err)
	}

	return &State{
		jwtPublicKey:  jwtPublicKey,
		jwtPrivateKey: jwtPrivateKey,
	}
}

func readJWTPrivateKey(path string) ([]byte, error) {
	key, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	return key, nil
}

func readJWTPublicKey(path string) ([]byte, error) {
	key, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	return key, nil
}

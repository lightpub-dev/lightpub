package auth

import "os"

type State struct {
	jwtPublicKey  []byte
	jwtPrivateKey []byte
}

func NewStateFromEnv() *State {
	jwtPublicKey, err := readJWTPublicKey()
	if err != nil {
		panic(err)
	}

	jwtPrivateKey, err := readJWTPrivateKey()
	if err != nil {
		panic(err)
	}

	return &State{
		jwtPublicKey:  jwtPublicKey,
		jwtPrivateKey: jwtPrivateKey,
	}
}

func readJWTPrivateKey() ([]byte, error) {
	path, exists := os.LookupEnv("JWT_SECRET_KEY_FILE")
	if !exists {
		panic("JWT_SECRET_KEY_FILE not set")
	}

	key, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	return key, nil
}

func readJWTPublicKey() ([]byte, error) {
	path, exists := os.LookupEnv("JWT_PUBLIC_KEY_FILE")
	if !exists {
		panic("JWT_PUBLIC_KEY_FILE not set")
	}

	key, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	return key, nil
}

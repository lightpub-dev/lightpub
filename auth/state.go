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

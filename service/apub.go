package service

import (
	"context"
	"fmt"
	"net/http"

	"github.com/go-fed/httpsig"
	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) isAllowedToSend(ctx context.Context, targetURL string) bool {
	// TODO: localhost check
	// TODO: blocked instnace check
	return true
}

func (s *State) AuthorizeWithHttpSig(ctx context.Context, r *http.Request) (types.UserID, error) {
	user, err := s.verifyHttpSig(ctx, r)
	if err != nil {
		return types.UserID{}, NewServiceErrorWithCause(http.StatusUnauthorized, "http signature validation failed", err)
	}

	// user is non-nil
	return user.Basic.ID, nil
}

func (s *State) verifyHttpSig(ctx context.Context, r *http.Request) (*types.ApubUser, error) {
	verifier, err := httpsig.NewVerifier(r)
	if err != nil {
		return nil, err
	}
	pubKeyId := verifier.KeyId()

	var algo httpsig.Algorithm = httpsig.RSA_SHA256

	user, err := s.findApubUserByKeyID(ctx, pubKeyId)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, fmt.Errorf("user not found for key ID: %s", pubKeyId)
	}
	pubKey := user.Apub.PublicKey()
	// The verifier will verify the Digest in addition to the HTTP signature
	err = verifier.Verify(pubKey, algo)
	if err != nil {
		return nil, err
	}
	return user, nil
}

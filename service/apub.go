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

package service

import (
	"context"
	"fmt"
	"net/http"

	"github.com/go-fed/httpsig"
	"github.com/lightpub-dev/lightpub/apub"
	"github.com/lightpub-dev/lightpub/types"
)

var (
	ErrUnsupportedActivityType = NewServiceError(http.StatusBadRequest, "unsupported activity type")
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

func (s *State) ReceiveActivity(ctx context.Context, signerID types.UserID, activity apub.InboxActivity) error {
	switch act := activity.(type) {
	case apub.FollowActivity:
		return s.handleFollowActivity(ctx, act)
	default:
		return ErrUnsupportedActivityType
	}
}

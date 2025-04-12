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

package apub

import (
	"bytes"
	"context"
	"crypto"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"time"
)

type Requester struct {
	// requester
	client  *http.Client
	baseURL *url.URL
}

func NewRequester(baseURL *url.URL, devMode bool) *Requester {
	tr := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: devMode},
	}
	client := &http.Client{Transport: tr}

	return &Requester{
		client:  client,
		baseURL: baseURL,
	}
}

func (s *Requester) MyDomain() string {
	return s.baseURL.Host
}

func (s *Requester) queueActivityInternal(ctx context.Context, q queuedActivity) error {
	// TODO: queue activtiy
	// TODO: execute immediately for now

	send := func() error {
		sendCtx := context.Background()
		privateKey, err := q.Signer.PrivateKeyObject()
		if err != nil {
			return fmt.Errorf("failed to get private key object: %w", err)
		}

		targetURL, err := url.Parse(q.Inbox)
		if err != nil {
			return fmt.Errorf("failed to parse inbox URL: %w", err)
		}
		targetHost := targetURL.Host

		body := bytes.NewBufferString(q.Activity)
		req, err := http.NewRequestWithContext(sendCtx, http.MethodPost, q.Inbox, body)
		if err != nil {
			return fmt.Errorf("failed to construct request: %w", err)
		}
		req.Header.Set("Content-Type", "application/activity+json")
		req.Header.Set("Accept", "application/activity+json")
		req.Header.Set("Date", time.Now().Format(http.TimeFormat))
		req.Header.Set("Host", targetHost)
		signRequest(privateKey, q.Signer.KeyID, req, body.Bytes())

		resp, err := s.client.Do(req)
		if err != nil {
			return fmt.Errorf("failed to send request: %w", err)
		}
		defer resp.Body.Close()
		success := resp.StatusCode >= 200 && resp.StatusCode < 300
		if !success {
			return fmt.Errorf("send activity returned error: (%d) %s", resp.StatusCode, resp.Status)
		}
		slog.DebugContext(sendCtx, "activity sent", "inbox", q.Inbox, "signer", q.Signer.ID)

		return nil
	}
	go func() {
		if err := send(); err != nil {
			slog.WarnContext(ctx, "failed to send activity", "error", err)
		}
	}()

	return nil
}

// queuedActivity represents an activity that has been queued for delivery.
// It contains the activity data, the recipients, and information required for signing.
type queuedActivity struct {
	// Activity in JSON format
	Activity string        `json:"activity"`
	Inbox    string        `json:"inbox"`
	Signer   minimalSigner `json:"signer"`
}

type minimalSigner struct {
	ID         string `json:"id"`
	PrivateKey string `json:"privkey"`
	PublicKey  string `json:"pubkey"`
	KeyID      string `json:"keyid"`
}

func (s minimalSigner) PrivateKeyObject() (crypto.PrivateKey, error) {
	block, _ := pem.Decode([]byte(s.PrivateKey))
	if block == nil {
		return nil, fmt.Errorf("failed to decode private key PEM")
	}
	privateKey, err := x509.ParsePKCS8PrivateKey(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("failed to parse private key: %w", err)
	}
	return privateKey, nil
}

func newMinimalSignerFromActor(actor Actor) (minimalSigner, error) {
	privateKey := actor.PrivateKey()
	if privateKey == nil {
		return minimalSigner{}, fmt.Errorf("actor has no private key")
	}
	privateKeyPem, err := x509.MarshalPKCS8PrivateKey(privateKey)
	if err != nil {
		return minimalSigner{}, fmt.Errorf("failed to serialize private key: %w", err)
	}
	privateKeyStr := pem.EncodeToMemory(&pem.Block{
		Type:  "PRIVATE KEY",
		Bytes: privateKeyPem,
	})

	publicKey := actor.PublicKey()
	if publicKey == nil {
		return minimalSigner{}, fmt.Errorf("actor has no public key")
	}
	publicKeyPem, err := x509.MarshalPKIXPublicKey(publicKey)
	if err != nil {
		return minimalSigner{}, fmt.Errorf("failed to serialize public key: %w", err)
	}
	publicKeyStr := pem.EncodeToMemory(&pem.Block{
		Type:  "PUBLIC KEY",
		Bytes: publicKeyPem,
	})

	return minimalSigner{
		ID:         actor.ID(),
		PrivateKey: string(privateKeyStr),
		PublicKey:  string(publicKeyStr),
		KeyID:      actor.KeyID(),
	}, nil
}

// QueueActivity queues an activity for delivery.
// The activity should be unsigned.
func (s *Requester) QueueActivity(ctx context.Context, activity any, signer Actor, targetInboxes []string) error {
	if s == nil {
		return nil
	}

	signerInfo, err := newMinimalSignerFromActor(signer)
	if err != nil {
		return fmt.Errorf("failed to create signer info: %w", err)
	}

	activityData, err := json.Marshal(activity)
	if err != nil {
		return fmt.Errorf("failed to marshal activity: %w", err)
	}

	// unique inboxes
	inboxSet := make(map[string]struct{})
	for _, inbox := range targetInboxes {
		inboxSet[inbox] = struct{}{}
	}

	// queue each inbox
	for inbox := range inboxSet {
		data := queuedActivity{
			Activity: string(activityData),
			Inbox:    inbox,
			Signer:   signerInfo,
		}
		if err := s.queueActivityInternal(ctx, data); err != nil {
			return fmt.Errorf("failed to queue activity: %w", err)
		}
	}

	return nil
}

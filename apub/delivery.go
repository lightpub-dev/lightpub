package apub

import (
	"bytes"
	"context"
	"crypto"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"log/slog"
	"net/http"
)

type DeliveryState struct {
	// requester
	client *http.Client
}

func NewDeliveryState() *DeliveryState {
	return &DeliveryState{
		client: http.DefaultClient,
	}
}

func (s *DeliveryState) queueActivityInternal(ctx context.Context, q queuedActivity) error {
	// TODO: queue activtiy
	// TODO: execute immediately for now

	send := func() error {
		privateKey, err := q.Signer.PrivateKeyObject()
		if err != nil {
			return fmt.Errorf("failed to get private key object: %w", err)
		}

		body := bytes.NewBufferString(q.Activity)
		req, err := http.NewRequestWithContext(ctx, http.MethodPost, q.Inbox, body)
		if err != nil {
			return fmt.Errorf("failed to construct request: %w", err)
		}
		req.Header.Set("Content-Type", "application/activity+json")
		req.Header.Set("Accept", "application/activity+json")
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
		Type:  "RSA PRIVATE KEY",
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
		Type:  "RSA PUBLIC KEY",
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
func (s *DeliveryState) QueueActivity(ctx context.Context, activity any, signer Actor, targetInboxes []string) error {
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

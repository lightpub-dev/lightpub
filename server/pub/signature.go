package pub

import (
	"fmt"
	"net/http"
	"time"

	"github.com/go-fed/httpsig"
	"github.com/lightpub-dev/lightpub/db"
)

type SignatureService struct {
	key KeyResolveService
}

func ProvideSignatureService(key KeyResolveService) *SignatureService {
	return &SignatureService{key}
}

func attachRequiredHeaders(req *http.Request) error {
	if req.Header.Get("date") == "" {
		req.Header.Set("date", time.Now().UTC().Format(http.TimeFormat))
	}
	if req.Header.Get("content-type") == "" {
		req.Header.Set("content-type", ActivityJsonType)
	}
	if req.Header.Get("host") == "" {
		url, err := req.URL.Parse(req.URL.String())
		if err != nil {
			return fmt.Errorf("failed to parse request URL: %w", err)
		}
		req.Header.Set("host", url.Host)
	}
}

func attachSignatureToAuthorization(req *http.Request) {
	// misskey requires this
	req.Header.Set("authorization", "Signature "+req.Header.Get("signature"))
}

func (s *SignatureService) Sign(actor *db.User, req *http.Request, body []byte) error {
	prefs := []httpsig.Algorithm{httpsig.RSA_SHA512, httpsig.RSA_SHA256}
	digestAlgorithm := httpsig.DigestSha256

	headersToSign := []string{httpsig.RequestTarget, "date", "content-type", "host"}
	if body != nil {
		headersToSign = append(headersToSign, "digest")
	}

	if err := attachRequiredHeaders(req); err != nil {
		return fmt.Errorf("failed to attach required headers: %w", err)
	}

	signer, _, err := httpsig.NewSigner(prefs, digestAlgorithm, headersToSign, httpsig.Signature, 3600)
	if err != nil {
		return fmt.Errorf("failed to create signer: %w", err)
	}

	privateKey, err := s.key.ResolvePrivateKey(actor)
	if err != nil {
		return fmt.Errorf("failed to resolve private key: %w", err)
	}

	if err := signer.SignRequest(privateKey.Key, privateKey.KeyID, req, body); err != nil {
		return fmt.Errorf("failed to sign request: %w", err)
	}

	attachSignatureToAuthorization(req)
	return nil
}

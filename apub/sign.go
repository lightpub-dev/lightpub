package apub

import (
	"crypto"
	"net/http"

	"github.com/go-fed/httpsig"
)

const (
	signatureExpiration = 60 * 60 // 1 hour
)

// signRequest signs the HTTP request using the provided private key and public key ID.
// body may be nil if the request body is not present.
func signRequest(privateKey crypto.PrivateKey, pubKeyId string, r *http.Request, body []byte) error {
	prefs := []httpsig.Algorithm{httpsig.RSA_SHA512, httpsig.RSA_SHA256}
	digestAlgorithm := httpsig.DigestSha256
	// The "Date" and "Digest" headers must already be set on r, as well as r.URL.
	headersToSign := []string{httpsig.RequestTarget, "date", "digest"}
	signer, _, err := httpsig.NewSigner(prefs, digestAlgorithm, headersToSign, httpsig.Signature, signatureExpiration)
	if err != nil {
		return err
	}
	// If r were a http.ResponseWriter, call SignResponse instead.
	return signer.SignRequest(privateKey, pubKeyId, r, body)
}

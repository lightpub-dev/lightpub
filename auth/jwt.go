package auth

import (
	"fmt"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/types"
)

const (
	jwtIssuer   = "lightpub"
	jwtDuration = 24 * time.Hour
)

type Claims struct {
	Exp int64  `json:"exp"`
	Iat int64  `json:"iat"`
	Iss string `json:"iss"`
	Nbf int64  `json:"nbf"`
	Sub string `json:"sub"`
}

func (c Claims) GetExpirationTime() (*jwt.NumericDate, error) {
	return jwt.NewNumericDate(time.Unix(c.Exp, 0)), nil
}

func (c Claims) GetIssuedAt() (*jwt.NumericDate, error) {
	return jwt.NewNumericDate(time.Unix(c.Iat, 0)), nil
}

func (c Claims) GetNotBefore() (*jwt.NumericDate, error) {
	return jwt.NewNumericDate(time.Unix(c.Nbf, 0)), nil
}

func (c Claims) GetIssuer() (string, error) {
	return c.Iss, nil
}

func (c Claims) GetSubject() (string, error) {
	return c.Sub, nil
}

func (c Claims) GetAudience() (jwt.ClaimStrings, error) {
	return nil, nil
}

func (s *State) GenerateJWT(userID types.UserID) (string, error) {
	now := time.Now()
	iat := now.Unix()
	exp := now.Add(jwtDuration).Unix()

	userIDStr := userID.String()

	claims := Claims{
		Exp: exp,
		Iat: iat,
		Iss: jwtIssuer,
		Nbf: iat,
		Sub: userIDStr,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
	signKey, err := jwt.ParseRSAPrivateKeyFromPEM(s.jwtPrivateKey)
	if err != nil {
		return "", err
	}

	return token.SignedString(signKey)
}

func (s *State) VerifyJWT(tokenString string) (Claims, error) {
	var claims Claims

	verifyKey, err := jwt.ParseRSAPublicKeyFromPEM(s.jwtPublicKey)
	if err != nil {
		return claims, err
	}

	token, err := jwt.ParseWithClaims(tokenString, &claims, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodRSA); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return verifyKey, nil
	})

	if err != nil || !token.Valid {
		return claims, failure.NewError(http.StatusUnauthorized, "invalid token")
	}

	return claims, nil
}

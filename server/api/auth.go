package api

import (
	"database/sql"
	"errors"
	"os"
	"os/exec"
	"path"
	"strconv"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/utils"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

const (
	ContextUserID   = "user_id"
	ContextUsername = "username"
	ContextAuthed   = "authed"
)

// echo auth middleware
func (h *Handler) AuthMiddleware(allowUnauthed bool) func(echo.HandlerFunc) echo.HandlerFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			// read Authorization header
			// expect Bearer token
			// assign Bearer token to bearer variable
			// if bearer is empty, return 401
			header := c.Request().Header.Get("Authorization")
			if header == "" {
				if allowUnauthed {
					c.Set(ContextAuthed, false)
					return next(c)
				}
				return c.String(401, "Authorization header is missing")
			}

			// check if bearer
			// if not, return 401
			if header[:6] != "Bearer" {
				return c.String(401, "Authorization must be Bearer token")
			}

			token := header[7:]

			// validate it
			var user db.UserToken
			result := h.DB.Model(&db.UserToken{}).Where("token = ?", token).Joins("User").Where("User.host IS NULL").First(&user)
			// if not found, return 401
			if errors.Is(result.Error, gorm.ErrRecordNotFound) {
				return c.String(401, "Invalid auth token")
			}
			if result.Error != nil {
				c.Logger().Error(result.Error)
				return c.String(500, "Internal Server Error")
			}

			// if found, set user_id in context
			c.Set(ContextAuthed, true)
			c.Set(ContextUserID, user.User.ID)
			c.Set(ContextUsername, user.User.Username)

			// call next handler
			return next(c)
		}
	}
}

func (h *Handler) PostLogin(c echo.Context) error {
	var req struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}

	// read request body
	err := c.Bind(&req)
	if err != nil {
		return c.String(400, "Bad Request")
	}

	var user db.User
	result := h.DB.First(&user, "username = ? AND host IS NULL", req.Username)
	err = result.Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return c.String(401, "bad auth")
		}
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// check password
	if bcrypt.CompareHashAndPassword([]byte(user.Bpasswd), []byte(req.Password)) != nil {
		return c.String(401, "bad auth")
	}

	// generate token
	token, err := generateToken()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// insert token
	result = h.DB.Create(&db.UserToken{
		UserID: user.ID,
		Token:  token.String(),
	})
	err = result.Error
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// return token
	return c.JSON(200, map[string]interface{}{
		"token": token.String(),
	})
}

type keyPair struct {
	PrivateKey string
	PublicKey  string
}

func generateKeyPair() (keyPair, error) {
	// create a temporary directory
	tmpDir, err := os.MkdirTemp("", "lightpub-keygen")
	if err != nil {
		return keyPair{}, err
	}
	// clean up directory after finish
	defer os.RemoveAll(tmpDir)
	// chmod 700
	if err := os.Chmod(tmpDir, 0700); err != nil {
		return keyPair{}, err
	}

	// generate key pair using OpenSSL
	keyPairFile := path.Join(tmpDir, "keypair.pem")
	bits := 2048
	cmd := exec.Command("openssl", "genrsa", "-out", keyPairFile, strconv.Itoa(bits))
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}

	// extract public key
	publicKeyFile := path.Join(tmpDir, "public.pem")
	cmd = exec.Command("openssl", "rsa", "-in", keyPairFile, "-pubout", "-out", publicKeyFile)
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}
	// read public key
	publicKey, err := os.ReadFile(publicKeyFile)
	if err != nil {
		return keyPair{}, err
	}

	// export private key as pkcs8
	privateKeyFile := path.Join(tmpDir, "private.pem")
	cmd = exec.Command("openssl", "pkcs8", "-topk8", "-inform", "PEM", "-outform", "PEM", "-nocrypt", "-in", keyPairFile, "-out", privateKeyFile)
	if err := cmd.Run(); err != nil {
		return keyPair{}, err
	}
	// read private key
	privateKey, err := os.ReadFile(privateKeyFile)
	if err != nil {
		return keyPair{}, err
	}

	publicKeyString := string(publicKey)
	privateKeyString := string(privateKey)

	return keyPair{
		PrivateKey: privateKeyString,
		PublicKey:  publicKeyString,
	}, nil
}

func (h *Handler) PostRegister(c echo.Context) error {
	var req struct {
		Username string `json:"username" validate:"alphanum,max=60,min=1"`
		Nickname string `json:"nickname" validate:"max=200,min=1"`
		Password string `json:"password" validate:"min=4"`
	}

	// read request body
	err := c.Bind(&req)
	if err != nil {
		return c.String(400, "Bad Request")
	}

	// check if username is taken
	tx := h.DB.Begin()
	defer tx.Rollback()

	var count int64
	result := tx.Model(&db.User{}).Where("username = ?", req.Username).Count(&count)
	err = result.Error
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if count > 0 {
		return c.String(409, "Username already taken")
	}

	// hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	userId, err := utils.GenerateUUID()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	keyPair, err := generateKeyPair()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	user := db.User{
		ID:       db.UUID(userId),
		Username: req.Username,
		Nickname: req.Nickname,
		Bpasswd:  string(hashedPassword),
		Host:     sql.NullString{}, // null for local user
		PublicKey: sql.NullString{
			String: keyPair.PublicKey,
			Valid:  true,
		},
		PrivateKey: sql.NullString{
			String: keyPair.PrivateKey,
			Valid:  true,
		},
		Bio: "",
	}

	result = tx.Create(&user)
	err = result.Error
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// commit
	err = tx.Commit().Error
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.NoContent(201)
}

func generateToken() (uuid.UUID, error) {
	return uuid.NewRandom()
}

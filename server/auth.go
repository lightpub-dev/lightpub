package main

import (
	"database/sql"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/utils"
	"golang.org/x/crypto/bcrypt"
)

const (
	ContextUserID   = "user_id"
	ContextUsername = "username"
	ContextAuthed   = "authed"
)

// echo auth middleware
func authMiddleware(allowUnauthed bool) func(echo.HandlerFunc) echo.HandlerFunc {
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
			var users []models.User
			err := db.Select(&users, "SELECT BIN_TO_UUID(u.id) AS id,u.username FROM User u INNER JOIN UserToken ut ON u.id=ut.user_id WHERE ut.token=? AND u.is_local=1", token)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "Internal Server Error")
			}

			// if not found, return 401
			if len(users) == 0 {
				return c.String(401, "Invalid auth token")
			}

			// if found, set user_id in context
			c.Set(ContextAuthed, true)
			c.Set(ContextUserID, users[0].ID)
			c.Set(ContextUsername, users[0].Username)

			// call next handler
			return next(c)
		}
	}
}

func postLogin(c echo.Context) error {
	var req struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}

	// read request body
	err := c.Bind(&req)
	if err != nil {
		return c.String(400, "Bad Request")
	}

	var user models.User
	err = db.Get(&user, "SELECT * FROM User WHERE username=? AND is_local=1", req.Username)
	if err != nil {
		if err == sql.ErrNoRows {
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
	_, err = db.Exec("INSERT INTO UserToken (user_id, token) VALUES (?, ?)", user.ID, token)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// return token
	return c.JSON(200, map[string]interface{}{
		"token": token.String(),
	})
}

func postRegister(c echo.Context) error {
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
	tx, err := db.Beginx()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}
	defer tx.Rollback()

	var count int
	err = tx.Get(&count, "SELECT COUNT(*) FROM User WHERE username=? FOR UPDATE", req.Username)
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

	userId, err := utils.GenerateUUIDString()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	user := models.User{
		ID:       userId,
		Username: req.Username,
		Nickname: req.Nickname,
		Bpasswd:  string(hashedPassword),
		Host:     "",
	}

	_, err = tx.NamedExec("INSERT INTO User (id,username,nickname,bpassword,host) VALUES (UUID_TO_BIN(:id),:username,:nickname,:bpassword,:host)", user)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// commit
	err = tx.Commit()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.NoContent(201)
}

func generateToken() (uuid.UUID, error) {
	return uuid.NewRandom()
}

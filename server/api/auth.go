package api

import (
	"errors"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/users"
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
			service := initializeUserLoginService(c, h)
			user, err := service.TokenAuth(token)
			if err != nil {
				c.Logger().Error(err)
				return c.String(http.StatusInternalServerError, "Internal Server Error")
			}
			if user == nil {
				if allowUnauthed {
					c.Set(ContextAuthed, false)
					return next(c)
				}
				return c.String(401, "Unauthorized")
			}

			// if found, set user_id in context
			c.Set(ContextAuthed, true)
			c.Set(ContextUserID, user.UserID)
			c.Set(ContextUsername, user.Username)

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

	service := initializeUserLoginService(c, h)
	token, err := service.Login(req.Username, req.Password)
	if err != nil {
		if errors.Is(err, users.ErrBadAuth) {
			return c.String(http.StatusUnauthorized, "Bad auth")
		}
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "Internal Server Error")
	}

	// return token
	return c.JSON(200, map[string]interface{}{
		"token": token,
	})
}

func (h *Handler) PostRegister(c echo.Context) error {
	var req struct {
		Username string `json:"username" validate:"ascii,max=60,min=1"`
		Nickname string `json:"nickname" validate:"max=200,min=1"`
		Password string `json:"password" validate:"min=4"`
	}

	// read request body
	err := c.Bind(&req)
	if err != nil {
		return c.String(400, "Bad Request")
	}

	// validate request body
	err = validate.Struct(req)
	if err != nil {
		c.Logger().Debug(err)
		return c.String(400, "Bad body format")
	}

	service := initializeUserCreateService(c, h)
	if err := service.CreateUser(users.UserCreateRequest{
		Username: req.Username,
		Nickname: req.Nickname,
		Password: req.Password,
	}); err != nil {
		if errors.Is(err, users.ErrUsernameTaken) {
			return c.String(409, "Username is taken")
		}
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "Internal Server Error")
	}

	return c.NoContent(201)
}

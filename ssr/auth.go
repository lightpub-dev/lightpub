package main

import (
	"encoding/json"
	"net/http"

	"github.com/labstack/echo/v4"
)

const (
	CookieLoginToken = "LoginToken"
)

type authForm struct {
	Error string
}

func (h *handler) registerView(c echo.Context) error {
	return c.Render(http.StatusOK, "register.html", authForm{})
}

func (h *handler) loginView(c echo.Context) error {
	return c.Render(http.StatusOK, "login.html", authForm{})
}

type registerPostForm struct {
	Username string `form:"username" json:"username"`
	Nickname string `form:"nickname" json:"nickname"`
	Password string `form:"password" json:"password"`
}

func (h *handler) registerPost(c echo.Context) error {
	var form registerPostForm
	if err := c.Bind(&form); err != nil {
		return err
	}

	if form.Username == "" || form.Nickname == "" || form.Password == "" {
		return c.Render(http.StatusBadRequest, "register.html", authForm{
			Error: "All fields are required",
		})
	}

	resp, err := h.req.Post("/register", form)
	if err != nil {
		return c.Render(http.StatusInternalServerError, "register.html", authForm{
			Error: "Could not connect to server"})
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return c.Render(resp.StatusCode, "register.html", authForm{
			Error: "Failed to register",
		})
	}

	return c.Redirect(http.StatusSeeOther, "/login")
}

type loginPostForm struct {
	Username string `form:"username" json:"username"`
	Password string `form:"password" json:"password"`
}

type loginAPIResponse struct {
	Token string `json:"token"`
}

func (h *handler) loginPost(c echo.Context) error {
	var form loginPostForm
	if err := c.Bind(&form); err != nil {
		return err
	}

	if form.Username == "" || form.Password == "" {
		return c.Render(http.StatusBadRequest, "login.html", authForm{
			Error: "All fields are required",
		})
	}

	resp, err := h.req.Post("/login", form)
	if err != nil {
		return c.Render(http.StatusInternalServerError, "login.html", authForm{
			Error: "Could not connect to server"})
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return c.Render(resp.StatusCode, "login.html", authForm{
			Error: "Failed to login",
		})
	}

	// read response
	var respBody loginAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&respBody); err != nil {
		return c.Render(http.StatusInternalServerError, "login.html", authForm{
			Error: "Failed to login",
		})
	}

	// set login token as cookie
	cookie := new(http.Cookie)
	cookie.Name = CookieLoginToken
	cookie.Value = respBody.Token
	cookie.HttpOnly = true
	cookie.Secure = true
	cookie.MaxAge = 60 * 60 * 24 * 7 // 1 week
	cookie.SameSite = http.SameSiteStrictMode
	c.SetCookie(cookie)

	return c.Redirect(http.StatusSeeOther, "/timeline")
}

func getToken(c echo.Context) string {
	cookie, err := c.Cookie(CookieLoginToken)
	if err != nil {
		return ""
	}
	return cookie.Value
}

package api

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/users"
	"github.com/lightpub-dev/lightpub/webfinger"
)

func (h *Handler) GetWebfinger(c echo.Context) error {
	// get resource parameter
	resource := c.QueryParam("resource")
	if resource == "" {
		return c.String(400, "invalid resource")
	}

	jsonResponse, err := webfinger.HandleWebfinger(c.Request().Context(), h.DB, resource)
	if err != nil {
		if err == webfinger.ErrBadFormat {
			return c.String(http.StatusBadRequest, "bad format")
		}
		if err == webfinger.ErrUnknown {
			return c.String(http.StatusUnprocessableEntity, "unknown")
		}
		if err == webfinger.ErrInvalidHost {
			return c.String(http.StatusUnprocessableEntity, "invalid host")
		}
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	return c.JSON(http.StatusOK, jsonResponse)
}

type nodeInfoRoot struct {
	Links []nodeInfoLink `json:"links"`
}

type nodeInfoLink struct {
	Rel  string `json:"rel"`
	Href string `json:"href"`
}

func (h *Handler) GetNodeInfo(c echo.Context) error {
	return c.JSON(http.StatusOK, nodeInfoRoot{
		Links: []nodeInfoLink{
			{
				Rel:  "http://nodeinfo.diaspora.software/ns/schema/2.1",
				Href: h.AbsoluteURL("/nodeinfo/2.1"),
			},
			{
				Rel:  "http://nodeinfo.diaspora.software/ns/schema/2.0",
				Href: h.AbsoluteURL("/nodeinfo/2.0"),
			},
		},
	})
}

func (h *Handler) NodeInfo2(c echo.Context, version string) error {
	userCount, err := users.CountLocalUsers(c.Request().Context(), h.MakeDB())
	if err != nil {
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	return c.JSON(http.StatusOK, map[string]interface{}{
		"version": version,
		"software": map[string]interface{}{
			"name":       "lightpub",
			"version":    "0.1", // TODO
			"repository": "https://github.com/lightpub-dev/lightpub",
		},
		"protocol": []string{
			"activitypub",
		},
		"services": map[string]interface{}{
			"inbound":  []string{},
			"outbound": []string{},
		},
		"openRegistrations": true, // TODO
		"usage": map[string]interface{}{
			"users": map[string]interface{}{
				"total": userCount, // TODO
			},
		},
		"metadata": map[string]interface{}{
			"nodeName":        "lightpub",                          // TODO
			"nodeDescription": "A light-weight ActivityPub server", // TODO
		},
	})
}

func (h *Handler) Nodeinfo20(c echo.Context) error {
	return h.NodeInfo2(c, "2.0")
}

func (h *Handler) Nodeinfo21(c echo.Context) error {
	return h.NodeInfo2(c, "2.1")
}

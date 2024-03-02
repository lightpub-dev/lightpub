package api

import (
	"bytes"
	"html/template"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/webfinger"
)

var (
	hostMetaTemplate = template.Must(template.ParseFiles("templates/host-meta.xml"))
)

func (h *Handler) GetWebfinger(c echo.Context) error {
	// get resource parameter
	resource := c.QueryParam("resource")
	if resource == "" {
		return c.String(400, "invalid resource")
	}

	return webfinger.HandleWebfinger(c, h.DB, resource)
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
	userService := initializeUserFinderService(c, h)
	userCount, err := userService.CountLocalUsers()
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

func (h *Handler) GetHostMeta(c echo.Context) error {
	buf := new(bytes.Buffer)
	if err := hostMetaTemplate.Execute(buf, map[string]string{
		"baseUrl": h.BaseURL,
	}); err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}
	return c.Blob(http.StatusOK, "application/xrd+xml", buf.Bytes())
}

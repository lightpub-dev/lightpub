package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/labstack/echo/v4"
)

func isJsonLDRequested(c echo.Context) bool {
	accept := c.Request().Header.Get("Accept")
	possibleTypes := []string{
		"application/activity+json",
		"application/ld+json",
	}
	for _, t := range possibleTypes {
		if strings.Contains(accept, t) {
			return true
		}
	}
	return false
}

func ResponseActivityJson(c echo.Context, obj interface{}) error {
	b, err := json.Marshal(obj)
	if err != nil {
		return fmt.Errorf("error marshalling JSON: %w", err)
	}
	return c.Blob(http.StatusOK, "application/activity+json", b)
}

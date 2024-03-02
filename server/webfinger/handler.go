package webfinger

import (
	"errors"
	"strings"

	"github.com/labstack/echo/v4"
	"gorm.io/gorm"
)

var (
	ErrBadFormat = errors.New("bad format")
	ErrUnknown   = errors.New("unknown")
)

func extractResourceType(resource string) (string, string, error) {
	// split by colon
	// if there is no colon, return error
	// if there is a colon, return the first part
	parts := strings.SplitN(resource, ":", 2)
	if len(parts) != 2 {
		return "", "", errors.New("invalid resource")
	}
	return parts[0], parts[1], nil
}

func HandleWebfinger(c echo.Context, conn *gorm.DB, resource string) error {
	resourceType, specifier, err := extractResourceType(resource)
	if err != nil {
		return err
	}

	switch resourceType {
	case "acct":
		return handleAcct(c, conn, specifier)
	default:
		return c.String(400, "invalid resource type")
	}
}

package webfinger

import (
	"context"
	"errors"
	"strings"

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

func HandleWebfinger(ctx context.Context, conn *gorm.DB, resource string) (interface{}, error) {
	resourceType, specifier, err := extractResourceType(resource)
	if err != nil {
		return nil, err
	}

	switch resourceType {
	case "acct":
		return handleAcct(ctx, conn, specifier)
	default:
		return nil, ErrUnknown
	}
}

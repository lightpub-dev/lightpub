package webfinger

import (
	"errors"
	"strings"
)

func extractResourceType(resource string) (string, error) {
	// split by colon
	// if there is no colon, return error
	// if there is a colon, return the first part
	parts := strings.SplitN(resource, ":", 2)
	if len(parts) != 2 {
		return "", errors.New("invalid resource")
	}
	return parts[0], nil
}

func HandleWebfinger(resource string) (interface{}, error) {
	resourceType, err := extractResourceType(resource)
	if err != nil {
		return nil, err
	}

	switch resourceType {

	}
}

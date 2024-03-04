package api

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/k0kubun/pp/v3"
	"github.com/labstack/echo/v4"
)

var (
	pubAcceptedHeaders = []string{
		"application/activity+json",
		"application/ld+json",
	}
)

func contentTypeCheck(c echo.Context) error {
	contentType := c.Request().Header.Get("Content-Type")
	for _, t := range pubAcceptedHeaders {
		if strings.Contains(contentType, t) {
			return nil
		}
	}
	return c.NoContent(http.StatusUnsupportedMediaType)
}

func (h *Handler) inboxAccept(c echo.Context, accept vocab.ActivityStreamsAccept) error {
	followService := initializeUserFollowService(c, h)
	if err := followService.AcceptFollowRequest(accept); err != nil {
		return err
	}

	return nil
}

func (h *Handler) UserInbox(c echo.Context) error {
	if err := contentTypeCheck(c); err != nil {
		return err
	}

	reqBodyStream := c.Request().Body
	defer reqBodyStream.Close()
	reqBody, err := io.ReadAll(reqBodyStream)
	if err != nil {
		return c.String(http.StatusInternalServerError, "failed to read body")
	}
	var jsonMap map[string]interface{}
	err = json.Unmarshal(reqBody, &jsonMap)
	if err != nil {
		c.Logger().Debugf("failed to unmarshal json: %s", err.Error())
		return c.String(http.StatusBadRequest, "invalid body")
	}

	fmt.Println("inbox got:")
	fmt.Println(pp.Sprint(jsonMap))

	inboxAccept := func(ctx context.Context, accept vocab.ActivityStreamsAccept) error {
		return h.inboxAccept(c, accept)
	}

	resolver, err := streams.NewJSONResolver(inboxAccept)
	if err != nil {
		c.Logger().Errorf("failed to resolve json: %s", err.Error())
		return c.String(http.StatusBadRequest, "invalid body")
	}

	err = resolver.Resolve(c.Request().Context(), jsonMap)
	if err != nil {
		c.Logger().Debugf("failed to resolve json: %s", err.Error())
		return c.String(http.StatusBadRequest, "failed to process your request")
	}

	return c.NoContent(http.StatusAccepted)
}

func (h *Handler) UserOutbox(c echo.Context) error {
	if err := contentTypeCheck(c); err != nil {
		return err
	}

	return c.NoContent(http.StatusMethodNotAllowed)
}

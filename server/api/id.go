package api

import (
	"net/url"

	"github.com/lightpub-dev/lightpub/db"
)

type IDGetter struct {
	h *Handler
}

func ProvideIDGetter(h *Handler) *IDGetter {
	return &IDGetter{h: h}
}

func (g *IDGetter) GetUserID(user *db.User, attribute string) (*url.URL, error) {
	if user.URI.Valid {
		// ignore attribute

		return url.Parse(user.URI.String)
	}

	userURL := g.h.BaseURL + "/user/" + user.ID.String()
	if attribute == "publicKey" {
		userURL += "#main-key"
	} else if attribute != "" {
		userURL += "/" + attribute
	}

	return url.Parse(userURL)
}

func (g *IDGetter) GetPostID(post *db.Post, attribute string) (*url.URL, error) {
	if post.URI.Valid {
		// ignore attribute
		return url.Parse(post.URI.String)
	}

	postURL := g.h.BaseURL + "/post/" + post.ID.String()
	if attribute != "" {
		postURL += "/" + attribute
	}

	return url.Parse(postURL)
}

func (g *IDGetter) GetFollowRequestID(req *db.UserFollowRequest) (*url.URL, error) {
	if req.URI.Valid {
		return url.Parse(req.URI.String)
	}

	reqURL := g.h.BaseURL + "/follow-request/" + req.ID.String()

	return url.Parse(reqURL)
}

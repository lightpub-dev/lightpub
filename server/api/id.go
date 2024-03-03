package api

import (
	"fmt"
	"net/url"
	"regexp"
	"strings"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
)

type IDGetter struct {
	h *Handler
}

func ProvideIDGetter(h *Handler) *IDGetter {
	return &IDGetter{h: h}
}

func (g *IDGetter) GetUserID(user *db.User, attribute pub.UserAttribute) (*url.URL, error) {
	if user.URI.Valid {
		// remote user
		switch attribute {
		case pub.UserAttributeInbox:
			if !user.Inbox.Valid {
				return nil, pub.ErrUserInboxNotSet
			}
			return url.Parse(user.Inbox.String)
		case pub.UserAttributeOutbox:
			if !user.Outbox.Valid {
				return nil, pub.ErrUserOutboxNotSet
			}
			return url.Parse(user.Outbox.String)
		case pub.UserAttributeSharedInbox:
			if !user.SharedInbox.Valid {
				return nil, pub.ErrUserSharedInboxNotSet
			}
			return url.Parse(user.SharedInbox.String)
		case "":
			return url.Parse(user.URI.String)
		default:
			return nil, fmt.Errorf("unsupported attribute: %s", attribute)
		}
	}

	// local user
	userURL := g.h.BaseURL + "/user/" + user.ID.String()
	if attribute == pub.UserAttributePublicKey {
		userURL += "#main-key"
	} else if attribute != "" {
		userURL += "/" + string(attribute)
	}

	return url.Parse(userURL)
}

func (g *IDGetter) GetPostID(post *db.Post, attribute string) (*url.URL, error) {
	if post.URI.Valid {
		// remote post
		return url.Parse(post.URI.String)
	}

	// local post
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

func (g *IDGetter) removePrefix(uri string) (string, bool) {
	if !strings.HasPrefix(uri, g.h.BaseURL) {
		return "", false
	}

	return uri[len(g.h.BaseURL):], true
}

var (
	ReLocalUserID          = regexp.MustCompile("^/user/([a-f0-9]+)$")
	ReLocalPostID          = regexp.MustCompile("^/post/([a-f0-9]+)$")
	ReLocalFollowRequestID = regexp.MustCompile("^/follow-request/([a-f0-9]+)$")
)

func matchAndGet(re *regexp.Regexp, target string) (string, bool) {
	matches := re.FindStringSubmatch(target)
	if matches == nil {
		return "", false
	}

	return matches[1], true
}

func (g *IDGetter) ExtractLocalUserID(uri string) (string, bool) {
	parts, ok := g.removePrefix(uri)
	if !ok {
		return "", false
	}

	return matchAndGet(ReLocalUserID, parts)
}

func (g *IDGetter) ExtractLocalPostID(uri string) (string, bool) {
	parts, ok := g.removePrefix(uri)
	if !ok {
		return "", false
	}

	return matchAndGet(ReLocalPostID, parts)
}

func (g *IDGetter) ExtractLocalFollowRequestID(uri string) (string, bool) {
	parts, ok := g.removePrefix(uri)
	if !ok {
		return "", false
	}

	return matchAndGet(ReLocalFollowRequestID, parts)
}

func (g *IDGetter) MyHostname() string {
	url, err := url.Parse(g.h.BaseURL)
	if err != nil {
		panic(err)
	}
	return url.Host
}

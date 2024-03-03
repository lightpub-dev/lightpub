package pub

import (
	"net/url"

	"github.com/lightpub-dev/lightpub/db"
)

type IDGetterService interface {
	GetUserID(user *db.User, attribute string) (*url.URL, error)
	GetPostID(post *db.Post, attribute string) (*url.URL, error)
	GetFollowRequestID(req *db.UserFollowRequest) (*url.URL, error)

	ExtractLocalUserID(uri string) (string, bool)
	ExtractLocalPostID(uri string) (string, bool)
	ExtractLocalFollowRequestID(uri string) (string, bool)

	MyHostname() string
}

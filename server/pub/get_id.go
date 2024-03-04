package pub

import (
	"errors"
	"net/url"

	"github.com/lightpub-dev/lightpub/db"
)

var (
	ErrUserInboxNotSet       = errors.New("user inbox not set or valid")
	ErrUserOutboxNotSet      = errors.New("user outbox not set or valid")
	ErrUserSharedInboxNotSet = errors.New("user shared inbox not set or valid")
)

type UserAttribute string

const (
	UserAttributeNone        UserAttribute = ""
	UserAttributeInbox       UserAttribute = "inbox"
	UserAttributeOutbox      UserAttribute = "outbox"
	UserAttributeSharedInbox UserAttribute = "sharedInbox"
	UserAttributePublicKey   UserAttribute = "publicKey"
)

var (
	ErrInvalidLocalID = errors.New("has our hostname, but not a valid ID")
)

type IDGetterService interface {
	GetUserID(user *db.User, attribute UserAttribute) (*url.URL, error)
	GetPostID(post *db.Post, attribute string) (*url.URL, error)
	GetFollowRequestID(req *db.UserFollowRequest) (*url.URL, error)

	ExtractLocalUserID(uri string) (string, error)
	ExtractLocalPostID(uri string) (string, error)
	ExtractLocalFollowRequestID(uri string) (string, error)

	MyHostname() string
}

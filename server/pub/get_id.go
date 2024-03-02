package pub

import "github.com/lightpub-dev/lightpub/db"

type IDGetterService interface {
	GetUserID(user *db.User, attribute string) (string, error)
	GetPostID(post *db.Post) (string, error)
}

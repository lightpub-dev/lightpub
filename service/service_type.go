package service

import "github.com/lightpub-dev/lightpub/types"

type SimpleUser struct {
	ID       types.UserID
	Username string
	Domain   string // Empty string means local server
	Nickname string
	Bio      string
	Avatar   *types.UploadID
}

func (s SimpleUser) Specifier() string {
	if s.Domain == EmptyDomain {
		return "@" + s.Username
	}
	return "@" + s.Username + "@" + s.Domain
}

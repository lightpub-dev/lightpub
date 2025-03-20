package types

const (
	EmptyDomain = "" // Empty string means local server
)

type SimpleUser struct {
	ID       UserID
	Username string
	Domain   string // Empty string means local server
	Nickname string
	Bio      string
	Avatar   *UploadID
}

func (s SimpleUser) Specifier() string {
	if s.Domain == EmptyDomain {
		return "@" + s.Username
	}
	return "@" + s.Username + "@" + s.Domain
}

func (s SimpleUser) IsRemote() bool {
	return s.Domain != EmptyDomain
}

func (s SimpleUser) IsLocal() bool {
	return s.Domain == EmptyDomain
}

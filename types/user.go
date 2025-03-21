package types

const (
	EmptyDomain = "" // Empty string means local server

	FollowStateNo      FollowState = 0
	FollowStateYes     FollowState = 1
	FollowStatePending FollowState = 2
)

type FollowState = int

type SimpleUser struct {
	ID       UserID
	Username string
	Domain   string // Empty string means local server
	Nickname string
	Bio      string
	Avatar   *UploadID
}

func makeSpecifier(username, domain string) string {
	if domain == EmptyDomain {
		return "@" + username
	}
	return "@" + username + "@" + domain
}

func (s SimpleUser) Specifier() string {
	return makeSpecifier(s.Username, s.Domain)
}

func (s SimpleUser) IsRemote() bool {
	return s.Domain != EmptyDomain
}

func (s SimpleUser) IsLocal() bool {
	return s.Domain == EmptyDomain
}

type DetailedUser struct {
	Basic   SimpleUser
	Details DetailedUserModel
}

type DetailedUserModel struct {
	FollowCount      uint64
	FollowerCount    uint64
	NoteCount        uint64
	AutoFollowAccept bool
	HideFollows      bool
	RemoteURL        *string
	RemoteViewURL    *string

	// when the user is logged in
	IsFollowing *FollowState
	IsFollowed  *FollowState
	IsBlocking  *bool
	IsBlocked   *bool
}

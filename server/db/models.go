package db

import (
	"time"

	"github.com/lightpub-dev/lightpub/db"
)

type User struct {
	ID        UUID      `gorm:"primaryKey"`
	Username  string    `gorm:"size:64"`
	Host      string    `gorm:"size:128"`
	Bpasswd   string    `gorm:"size:60"`
	Nickname  string    `gorm:"size:255"`
	URL       *string   `gorm:"size:512"`
	Inbox     *string   `gorm:"size:512"`
	Outbox    *string   `gorm:"size:512"`
	CreatedAt time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	UserLabels []UserLabelDB `gorm:"foreignKey:UserID"`
	Profile    *UserProfile  `gorm:"foreignKey:UserID"`
	Followers  []UserFollow  `gorm:"foreignKey:FolloweeID"`
	Following  []UserFollow  `gorm:"foreignKey:FollowerID"`
}

type FullUser struct {
	User
	Bio                 string
	Labels              []UserLabelDB
	IsFollowingByViewer bool
	Following           int64
	Followers           int64
	PostCount           int64
}

type UserLabelDB struct {
	UserID db.UUID `gorm:"primaryKey"`
	User   User
	Order  int `gorm:"primaryKey"`
	Key    string
	Value  string
}

func (UserLabelDB) TableName() string {
	return "user_labels"
}

type UserProfile struct {
	UserID UUID `gorm:"primaryKey"`
	User   User

	Bio string `gorm:"type:TEXT"`
}

type UserToken struct {
	ID     uint64 `gorm:"primaryKey"`
	UserID UUID
	User   User
	Token  string `gorm:"type:VARCHAR(64)"`
}

type Post struct {
	ID         UUID `gorm:"primaryKey"`
	PosterID   UUID
	Content    *string   `gorm:"type:LONGTEXT"` // Null when reposting
	InsertedAt time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`
	Privacy    string    `gorm:"type:ENUM('public','unlisted','follower','private')"` // enum treated as string
	ReplyToID  *UUID     // Nullable fields as pointers
	RepostOfID *UUID     // Nullable fields as pointers
	PollID     *UUID     // Nullable fields, assuming same type as ID

	Poster   User
	ReplyTo  *Post
	RepostOf *Post
	Poll     *PostPoll
}

type PostAttachment struct {
	ID      UUID
	PostID  UUID
	FileExt string `gorm:"size:128"`

	Post Post
}

type PostFavorite struct {
	ID         uint64 `gorm:"primaryKey"`
	PostID     UUID
	UserID     UUID
	IsBookmark bool

	Post Post
	User User
}

type PostHashtag struct {
	ID          uint64 `gorm:"primaryKey"`
	PostID      UUID
	HashtagName string `gorm:"size:255,index"`

	Post Post
}

type PostPoll struct {
	ID            UUID `gorm:"primaryKey"`
	AllowMultiple bool
	Due           *time.Time `gorm:"type:DATETIME(6)"`
}

type PostReaction struct {
	ID       uint64 `gorm:"primaryKey"`
	PostID   UUID
	Reaction string `gorm:"size:128"`
	UserID   UUID

	Post Post
	User User
}

type PollChoice struct {
	ID     uint64 `gorm:"primaryKey"`
	PollID UUID
	Title  string `gorm:"type:TEXT"`
	Count  int64  `gorm:"default:0"`

	Poll PostPoll
}

type PollVote struct {
	ID     uint64 `gorm:"primaryKey"`
	PollID UUID
	UserID UUID

	Poll PostPoll
	User User
}

type PostMention struct {
	ID           uint64 `gorm:"primaryKey"`
	PostID       UUID
	TargetUserID UUID

	Post       Post
	TargetUser User `gorm:"foreignKey:TargetUserID"`
}

type UserFollow struct {
	ID         uint64     `gorm:"primaryKey"`
	FollowerID UUID       `gorm:"uniqueIndex:idx_follower_followee"`
	FolloweeID UUID       `gorm:"uniqueIndex:idx_follower_followee"`
	CreatedAt  *time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	Follower User `gorm:"foreignKey:FollowerID"`
	Followee User `gorm:"foreignKey:FolloweeID"`
}

package db

import (
	"database/sql"
	"time"
)

type User struct {
	ID        UUID      `gorm:"primaryKey"`
	Username  string    `gorm:"size:64;uniqueIndex;not null"`
	Host      string    `gorm:"size:128;not null"`
	Bpasswd   string    `gorm:"size:60;not null"`
	Nickname  string    `gorm:"size:255;not null"`
	URL       *string   `gorm:"size:512"`
	Inbox     *string   `gorm:"size:512"`
	Outbox    *string   `gorm:"size:512"`
	CreatedAt time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`

	UserLabels []UserLabelDB `gorm:"foreignKey:UserID"`
	Profile    *UserProfile  `gorm:"foreignKey:UserID"`
	Followers  []UserFollow  `gorm:"foreignKey:FolloweeID"`
	Following  []UserFollow  `gorm:"foreignKey:FollowerID"`
	UserTokens []UserToken   `gorm:"foreignKey:UserID"`
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
	UserID UUID `gorm:"primaryKey"`
	User   User
	Order  int    `gorm:"primaryKey;not null"`
	Key    string `gorm:"type:TEXT;not null"`
	Value  string `gorm:"type:TEXT;not null"`
}

func (UserLabelDB) TableName() string {
	return "user_labels"
}

type UserProfile struct {
	UserID UUID `gorm:"primaryKey"`
	User   User

	Bio string `gorm:"type:TEXT;not null"`
}

type UserToken struct {
	ID     uint64 `gorm:"primaryKey"`
	UserID UUID
	User   User
	Token  string `gorm:"type:VARCHAR(64);not null"`
}

type Post struct {
	ID         UUID `gorm:"primaryKey"`
	PosterID   UUID
	Content    *string   `gorm:"type:LONGTEXT"` // Null when reposting
	InsertedAt time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	Privacy    string    `gorm:"type:ENUM('public','unlisted','follower','private');not null"` // enum treated as string
	ReplyToID  NullUUID  // Nullable fields as pointers
	RepostOfID NullUUID  // Nullable fields as pointers
	PollID     NullUUID  // Nullable fields, assuming same type as ID

	Poster   User
	ReplyTo  *Post
	RepostOf *Post
	Poll     *PostPoll
	Hashtags []PostHashtag `gorm:"foreignKey:PostID"`
	Mentions []PostMention `gorm:"foreignKey:PostID"`
}

type PostAttachment struct {
	ID      UUID
	PostID  UUID
	FileExt string `gorm:"size:128;not null"`

	Post Post
}

type PostFavorite struct {
	ID         uint64 `gorm:"primaryKey"`
	PostID     UUID   `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`
	UserID     UUID   `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`
	IsBookmark bool   `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`

	Post Post
	User User
}

type PostHashtag struct {
	ID          uint64 `gorm:"primaryKey"`
	PostID      UUID   `gorm:"not null"`
	HashtagName string `gorm:"size:255,index;not null"`

	Post Post
}

type PostPoll struct {
	ID            UUID         `gorm:"primaryKey"`
	AllowMultiple bool         `gorm:"not null"`
	Due           sql.NullTime `gorm:"type:DATETIME(6)"`
}

type PostReaction struct {
	ID       uint64 `gorm:"primaryKey"`
	PostID   UUID   `gorm:"not null"`
	Reaction string `gorm:"size:128;not null"`
	UserID   UUID   `gorm:"not null"`

	Post Post
	User User
}

type PollChoice struct {
	ID     uint64 `gorm:"primaryKey"`
	PollID UUID   `gorm:"not null"`
	Title  string `gorm:"type:TEXT;not null"`
	Count  int64  `gorm:"default:0;not null"`

	Poll PostPoll
}

type PollVote struct {
	ID     uint64 `gorm:"primaryKey"`
	PollID UUID   `gorm:"not null"`
	UserID UUID   `gorm:"not null"`

	Poll PostPoll
	User User
}

type PostMention struct {
	ID           uint64 `gorm:"primaryKey"`
	PostID       UUID   `gorm:"not null"`
	TargetUserID UUID   `gorm:"not null"`

	Post       Post
	TargetUser User `gorm:"foreignKey:TargetUserID"`
}

type UserFollow struct {
	ID         uint64    `gorm:"primaryKey"`
	FollowerID UUID      `gorm:"uniqueIndex:idx_follower_followee;not null"`
	FolloweeID UUID      `gorm:"uniqueIndex:idx_follower_followee;not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	Follower User `gorm:"foreignKey:FollowerID"`
	Followee User `gorm:"foreignKey:FolloweeID"`
}

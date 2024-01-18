package db

import "time"

type User struct {
	ID        UUID      `gorm:"primaryKey"`
	Username  string    `gorm:"size:64"`
	Host      string    `gorm:"size:128"`
	Bpasswd   string    `gorm:"size:60"`
	Nickname  string    `gorm:"size:255"`
	URL       *string   `gorm:"size:512"`
	Inbox     *string   `gorm:"size:512"`
	Outbox    *string   `gorm:"size:512"`
	CreatedAt time.Time `gorm:"autoCreateTime:nano;type=DATETIME(6)"`
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
	ID    uint64 `gorm:"primaryKey"`
	Order int
	Key   string
	Value string
}

type UserToken struct {
	ID     uint64 `gorm:"primaryKey"`
	UserID UUID
	Token  string `gorm:"type:VARCHAR(64)"`
}

type Post struct {
	ID         UUID `gorm:"primaryKey"`
	PosterID   UUID
	Content    *string   `gorm:"type:LONGTEXT"` // Null when reposting
	InsertedAt time.Time `gorm:"autoCreateTime:nano;type=DATETIME(6)"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type=DATETIME(6)"`
	Privacy    string    `gorm:"type:ENUM('public','unlisted','follower','private')"` // enum treated as string
	ReplyTo    *UUID
	RepostOf   *UUID
	PollID     *UUID
}

type PostAttachment struct {
	ID      UUID `gorm:"primaryKey"`
	PostID  UUID
	FileExt string `gorm:"size:128"`
}

type PostFavorite struct {
	ID         uint64 `gorm:"primaryKey"`
	PostID     UUID
	UserID     UUID
	IsBookmark bool
}

type PostHashtag struct {
	ID          uint64 `gorm:"primaryKey"`
	PostID      UUID
	HashtagName string `gorm:"size:255"`
}

type PostPoll struct {
	ID            UUID `gorm:"primaryKey"`
	AllowMultiple bool
	Due           *time.Time `gorm:"type:DATETIME(6)"`
}

type PostReaction struct {
	ID       uint64 `gorm:"primaryKey"`
	PostID   UUID
	Reaction string `db:"reaction"`
	UserID   string `db:"user_id"`
}

type PollChoice struct {
	ID     int64  `db:"id"`
	PollID string `db:"poll_id"`
	Title  string `db:"title"`
	Count  int64  `db:"count"`
}

type PollVote struct {
	ID     int64  `db:"id"`
	PollID string `db:"poll_id"`
	UserID string `db:"user_id"`
}

type PostMention struct {
	ID           int64  `db:"id"`
	PostID       string `db:"post_id"`
	TargetUserID string `db:"target_user_id"`
}

type UserFollow struct {
	ID         int64      `db:"id"`
	FollowerID string     `db:"follower_id"`
	FolloweeID string     `db:"followee_id"`
	CreatedAt  *time.Time `db:"created_at"`
}

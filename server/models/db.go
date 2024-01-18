package models

import "time"

type User struct {
	ID        []byte    `gorm:"primaryKey;type=BINARY(16)"`
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
	ID    uint64 `gorm:"primaryKey;type=BINARY(16)"`
	Order int
	Key   string
	Value string
}

type UserToken struct {
	ID     uint64 `gorm:"primaryKey"`
	UserID []byte `gorm:"type:BINARY(16)"`
	Token  string `gorm:"type:VARCHAR(64)"`
}

type Post struct {
	ID          []byte     `gorm:"primaryKey;type:BINARY(16)"`
	PosterID    []byte     `gorm:"type:BINARY(16)"`
	Content     *string    `gorm:"type:LONGTEXT"` // Null when reposting
	InsertedAt  time.Time  `gorm:"autoCreateTime:nano;type=DATETIME(6)"`
	CreatedAt   time.Time  `gorm:"autoCreateTime:nano;type=DATETIME(6)"`
	Privacy     string     `gorm:"type:ENUM('public','unlisted','follower','private')"` // enum treated as string
	ReplyTo     []byte     `gorm:"type:BINARY(16)"`                                     // Nullable fields as pointers
	RepostOf    *string    `db:"repost_of"`                                             // Nullable fields as pointers
	PollID      *string    `db:"poll_id"`                                               // Nullable fields, assuming same type as ID
	ScheduledAt *time.Time `db:"scheduled_at"`                                          // Nullable fields as pointers
}

type PostAttachment struct {
	ID      string `db:"id"`
	PostID  string `db:"post_id"`
	FileExt string `db:"file_ext"`
}

type PostFavorite struct {
	ID         int64  `db:"id"`
	PostID     string `db:"post_id"`
	UserID     string `db:"user_id"`
	IsBookmark bool   `db:"is_bookmark"`
}

type PostHashtag struct {
	ID          int64  `db:"id"`
	PostID      string `db:"post_id"`
	HashtagName string `db:"hashtag_name"`
}

type PostPoll struct {
	ID            string     `db:"id"`
	AllowMultiple bool       `db:"allow_multiple"`
	Due           *time.Time `db:"due"`
}

type PostReaction struct {
	ID       int    `db:"id"`
	PostID   string `db:"post_id"`
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

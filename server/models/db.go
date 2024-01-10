package models

import "time"

type User struct {
	ID       string  `db:"id"`
	Username string  `db:"username"`
	Host     string  `db:"host"`
	Bpasswd  string  `db:"bpassword"`
	Nickname string  `db:"nickname"`
	URL      *string `db:"url"`
	Inbox    *string `db:"inbox"`
	Outbox   *string `db:"outbox"`
	IsLocal  bool    `db:"is_local"`
}

type FullUser struct {
	User
	Bio                 string `db:"bio"`
	Labels              []UserLabelDB
	IsFollowingByViewer bool
	Following           int64 `db:"n_following"`
	Followers           int64 `db:"n_followers"`
	PostCount           int64 `db:"n_posts"`
}

type UserLabelDB struct {
	Order int    `db:"order"`
	Key   string `db:"key"`
	Value string `db:"value"`
}

type UserToken struct {
	ID     int64  `db:"id"`
	UserID string `db:"user_id"`
	Token  string `db:"token"`
}

type Post struct {
	ID          string     `db:"id" json:"id"`
	PosterID    string     `db:"poster_id" json:"poster_id"`
	Content     *string    `db:"content" json:"content"` // Null when reposting
	InsertedAt  time.Time  `db:"inserted_at"`
	CreatedAt   time.Time  `db:"created_at" json:"created_at"`
	Privacy     string     `db:"privacy" json:"privacy"` // enum treated as string
	ReplyTo     *string    `db:"reply_to"`               // Nullable fields as pointers
	RepostOf    *string    `db:"repost_of"`              // Nullable fields as pointers
	PollID      *string    `db:"poll_id"`                // Nullable fields, assuming same type as ID
	ScheduledAt *time.Time `db:"scheduled_at"`           // Nullable fields as pointers
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

package db

import (
	"database/sql"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

// ApubErrorReport represents error reports for ActivityPub operations
type ApubErrorReport struct {
	ID         int       `gorm:"primaryKey;autoIncrement"`
	Activity   string    `gorm:"type:text;not null"`
	ErrorMsg   string    `gorm:"type:text;not null"`
	ReceivedAt time.Time `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
}

// Note represents a post in the system
type Note struct {
	ID          types.NoteID   `gorm:"type:binary(16);primaryKey"`
	URL         sql.NullString `gorm:"type:varchar(512)"`
	ViewURL     sql.NullString `gorm:"type:varchar(512)"`
	AuthorID    types.UserID   `gorm:"type:binary(16);not null"`
	Content     sql.NullString `gorm:"type:text"`
	ContentType sql.NullString `gorm:"type:varchar(32)"`
	CreatedAt   time.Time      `gorm:"type:datetime(6);not null;index:idx_note_created_at,sort:desc"`
	InsertedAt  time.Time      `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
	UpdatedAt   sql.NullTime   `gorm:"type:datetime(6)"`
	DeletedAt   sql.NullTime   `gorm:"type:datetime(6)"`
	Visibility  string         `gorm:"type:enum('public','unlisted','follower','private');not null"`
	ReplyToID   *types.NoteID  `gorm:"type:binary(16);index:idx_note_reply_to_id"`
	RenoteOfID  *types.NoteID  `gorm:"type:binary(16);index:idx_note_renote_of_id"`
	Sensitive   bool           `gorm:"type:tinyint(1);not null;default:0"`
	FetchedAt   sql.NullTime   `gorm:"type:datetime(6)"`

	Author   User          `gorm:"foreignKey:AuthorID"`
	Likes    []NoteLike    `gorm:"foreignKey:NoteID"`
	Mentions []NoteMention `gorm:"foreignKey:NoteID"`
	Tags     []NoteTag     `gorm:"foreignKey:NoteID"`
	Uploads  []NoteUpload  `gorm:"foreignKey:NoteID"`
}

// NoteLike represents a like on a note
type NoteLike struct {
	ID        int          `gorm:"primaryKey;autoIncrement"`
	NoteID    types.NoteID `gorm:"type:binary(16);not null;uniqueIndex:idx_note_like_unique,priority:1"`
	Note      Note         `gorm:"foreignKey:NoteID"`
	UserID    types.UserID `gorm:"type:binary(16);not null;uniqueIndex:idx_note_like_unique,priority:2"`
	User      User         `gorm:"foreignKey:UserID"`
	IsPrivate bool         `gorm:"type:tinyint(1);not null;uniqueIndex:idx_note_like_unique,priority:3"`
	CreatedAt time.Time    `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
}

// NoteMention represents a mention in a note
type NoteMention struct {
	ID           int          `gorm:"primaryKey;autoIncrement"`
	NoteID       types.NoteID `gorm:"type:binary(16);not null;uniqueIndex:idx_note_mention_unique,priority:1"`
	Note         Note         `gorm:"foreignKey:NoteID"`
	TargetUserID types.UserID `gorm:"type:binary(16);not null;uniqueIndex:idx_note_mention_unique,priority:2"`
	TargetUser   User         `gorm:"foreignKey:TargetUserID"`
}

// NoteTag represents a tag attached to a note
type NoteTag struct {
	ID     int          `gorm:"primaryKey;autoIncrement"`
	NoteID types.NoteID `gorm:"type:binary(16);not null;uniqueIndex:idx_note_tag_unique,priority:1"`
	Note   Note         `gorm:"foreignKey:NoteID"`
	TagID  int          `gorm:"not null;uniqueIndex:idx_note_tag_unique,priority:2"`
	Tag    Tag          `gorm:"foreignKey:TagID"`
}

// NoteUpload represents an upload attached to a note
type NoteUpload struct {
	ID       int            `gorm:"primaryKey;autoIncrement"`
	NoteID   types.NoteID   `gorm:"type:binary(16);not null"`
	Note     Note           `gorm:"foreignKey:NoteID"`
	UploadID types.UploadID `gorm:"type:binary(16);not null"`
	Upload   Upload         `gorm:"foreignKey:UploadID"`
}

// Notification represents a notification for a user
type Notification struct {
	ID        int          `gorm:"primaryKey;autoIncrement"`
	UserID    types.UserID `gorm:"type:binary(16);not null;index:idx_notification_read_at,priority:1"`
	User      User         `gorm:"foreignKey:UserID"`
	Body      string       `gorm:"type:longtext;not null"`
	CreatedAt time.Time    `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
	ReadAt    sql.NullTime `gorm:"type:datetime(6);index:idx_notification_read_at,priority:2"`
}

type UnreadNotificationCount struct {
	UserID      types.UserID `gorm:"type:binary(16);not null"`
	UnreadCount uint64       `gorm:"type:int;not null"`
}

// PushNotification represents a push notification subscription
type PushNotification struct {
	ID        int          `gorm:"primaryKey;autoIncrement"`
	UserID    types.UserID `gorm:"type:binary(16);not null;uniqueIndex:idx_push_notification_unique,priority:1"`
	User      User         `gorm:"foreignKey:UserID"`
	Endpoint  string       `gorm:"type:varchar(512);not null;uniqueIndex:idx_push_notification_unique,priority:2"`
	P256dh    string       `gorm:"type:text;not null"`
	Auth      string       `gorm:"type:text;not null"`
	CreatedAt time.Time    `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
}

// RemotePublicKey represents a public key for a remote user
type RemotePublicKey struct {
	ID        int          `gorm:"primaryKey;autoIncrement"`
	OwnerID   types.UserID `gorm:"type:binary(16);not null"`
	Owner     User         `gorm:"foreignKey:OwnerID"`
	KeyID     string       `gorm:"type:varchar(512);not null;index:idx_remote_public_key_key_id_unique"`
	PublicKey string       `gorm:"type:text;not null"`
}

// Tag represents a tag that can be attached to notes
type Tag struct {
	ID    int       `gorm:"primaryKey;autoIncrement"`
	Name  string    `gorm:"type:varchar(256);not null"`
	Notes []NoteTag `gorm:"foreignKey:TagID"`
}

// Upload represents a file upload
type Upload struct {
	ID          types.UploadID `gorm:"type:binary(16);primaryKey"`
	Filename    sql.NullString `gorm:"type:varchar(64)"`
	URL         sql.NullString `gorm:"type:varchar(512)"`
	MimeType    string         `gorm:"type:varchar(255);not null"`
	UserAvatars []User         `gorm:"foreignKey:Avatar"`
	Notes       []NoteUpload   `gorm:"foreignKey:UploadID"`
}

// User represents a user in the system
type User struct {
	ID                types.UserID       `gorm:"type:binary(16);primaryKey"`
	Username          string             `gorm:"type:varchar(128);not null;uniqueIndex:user_unique_username,priority:1"`
	Domain            string             `gorm:"type:varchar(128);not null;uniqueIndex:user_unique_username,priority:2"`
	Password          sql.NullString     `gorm:"type:varchar(128)"`
	Nickname          string             `gorm:"type:varchar(255);not null"`
	Bio               string             `gorm:"type:text;not null"`
	Avatar            *types.UploadID    `gorm:"type:binary(16)"`
	AvatarUpload      *Upload            `gorm:"foreignKey:Avatar"`
	URL               sql.NullString     `gorm:"type:varchar(512);uniqueIndex:idx_user_url_unique"`
	Inbox             sql.NullString     `gorm:"type:varchar(512)"`
	SharedInbox       sql.NullString     `gorm:"type:varchar(512)"`
	Outbox            sql.NullString     `gorm:"type:varchar(512)"`
	PrivateKey        sql.NullString     `gorm:"type:text"`
	PublicKey         sql.NullString     `gorm:"type:text"`
	CreatedAt         sql.NullTime       `gorm:"type:datetime(6)"`
	FetchedAt         sql.NullTime       `gorm:"type:datetime(6)"`
	ViewURL           sql.NullString     `gorm:"type:varchar(512)"`
	FollowingURL      sql.NullString     `gorm:"column:following,type:varchar(512)"`
	FollowersURL      sql.NullString     `gorm:"column:followers,type:varchar(512)"`
	AutoFollowAccept  bool               `gorm:"type:tinyint(1);not null;default:1"`
	AuthExpiredAt     sql.NullTime       `gorm:"type:datetime(6)"`
	IsBot             bool               `gorm:"type:tinyint(1);not null;default:0"`
	IsAdmin           bool               `gorm:"type:tinyint(1);not null;default:0"`
	HideFollows       bool               `gorm:"type:tinyint(1);not null;default:0"`
	PreferredInbox    string             `gorm:"-"` // Generated column in DB
	Notes             []Note             `gorm:"foreignKey:AuthorID"`
	Likes             []NoteLike         `gorm:"foreignKey:UserID"`
	Mentions          []NoteMention      `gorm:"foreignKey:TargetUserID"`
	Notifications     []Notification     `gorm:"foreignKey:UserID"`
	PushNotifications []PushNotification `gorm:"foreignKey:UserID"`
	PublicKeys        []RemotePublicKey  `gorm:"foreignKey:OwnerID"`
	FollowedBy        []UserFollow       `gorm:"foreignKey:FollowedID"`
	Following         []UserFollow       `gorm:"foreignKey:FollowerID"`
	BlockedBy         []UserBlock        `gorm:"foreignKey:BlockedID"`
	Blocking          []UserBlock        `gorm:"foreignKey:BlockerID"`
}

// UserBlock represents a block relationship between users
type UserBlock struct {
	ID        int          `gorm:"primaryKey;autoIncrement"`
	BlockerID types.UserID `gorm:"type:binary(16);not null"`
	Blocker   User         `gorm:"foreignKey:BlockerID"`
	BlockedID types.UserID `gorm:"type:binary(16);not null"`
	Blocked   User         `gorm:"foreignKey:BlockedID"`
	BlockedAt time.Time    `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
}

// UserFollow represents a follow relationship between users
type UserFollow struct {
	ID         int            `gorm:"primaryKey;autoIncrement"`
	FollowerID types.UserID   `gorm:"type:binary(16);not null;uniqueIndex:user_follow_unique,priority:1"`
	Follower   User           `gorm:"foreignKey:FollowerID"`
	FollowedID types.UserID   `gorm:"type:binary(16);not null;uniqueIndex:user_follow_unique,priority:2"`
	Followed   User           `gorm:"foreignKey:FollowedID"`
	Pending    bool           `gorm:"type:tinyint(1);not null"`
	URL        sql.NullString `gorm:"type:varchar(512)"`
	CreatedAt  time.Time      `gorm:"type:datetime(6);not null;default:current_timestamp(6)"`
}

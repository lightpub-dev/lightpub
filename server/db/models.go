package db

import (
	"database/sql"
	"time"
)

type User struct {
	ID          UUID           `gorm:"primaryKey"`
	Username    string         `gorm:"size:64;uniqueIndex;not null"`
	Host        sql.NullString `gorm:"size:128"`
	Bpasswd     sql.NullString `gorm:"size:60"`
	Nickname    string         `gorm:"size:255;not null"`
	Bio         string         `gorm:"type:TEXT;not null"`
	AvatarID    NullUUID       `gorm:"type:VARCHAR(32);default:NULL"`
	Avatar      *UploadedFile  `gorm:"foreignKey:AvatarID"`
	URI         sql.NullString `gorm:"size:512"`
	SharedInbox sql.NullString `gorm:"size:512"`
	Inbox       sql.NullString `gorm:"size:512"`
	Outbox      sql.NullString `gorm:"size:512"`
	PrivateKey  sql.NullString `gorm:"type:TEXT"`
	PublicKey   sql.NullString `gorm:"type:TEXT"`
	CreatedAt   time.Time      `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`

	UserLabels []UserLabelDB `gorm:"foreignKey:UserID"`
	Followers  []UserFollow  `gorm:"foreignKey:FolloweeID"`
	Following  []UserFollow  `gorm:"foreignKey:FollowerID"`
	UserTokens []UserToken   `gorm:"foreignKey:UserID"`
	Key        *UserKey      `gorm:"foreignKey:OwnerID"`
	RemoteInfo *RemoteUser   `gorm:"foreignKey:UserID"`
}

type UserKey struct {
	ID        string    `gorm:"primaryKey;size:512;not null"`
	OwnerID   UUID      `gorm:"not null"`
	PublicKey string    `gorm:"type:TEXT;not null"`
	UpdatedAt time.Time `gorm:"autoUpdateTime:nano;type:DATETIME(6);not null"`

	Owner *User `gorm:"foreignKey:OwnerID"`
}

type RemoteUser struct {
	UserID    UUID           `gorm:"primaryKey"`
	Following sql.NullString `gorm:"type:VARCHAR(512);default:NULL"`
	Followers sql.NullString `gorm:"type:VARCHAR(512);default:NULL"`
	Liked     sql.NullString `gorm:"type:VARCHAR(512);default:NULL"`
	FetchedAt time.Time      `gorm:"autoUpdateTime:nano;type:DATETIME(6);not null"`

	User *User `gorm:"foreignKey:UserID"`
}

type FullUser struct {
	User
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

type UserToken struct {
	ID         uint64 `gorm:"primaryKey"`
	UserID     UUID
	User       User
	Token      string    `gorm:"type:VARCHAR(64);not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	LastUsedAt time.Time `gorm:"autoUpdateTime:nano;type:DATETIME(6);not null"`
}

type Post struct {
	ID         UUID `gorm:"primaryKey"`
	PosterID   UUID
	Content    sql.NullString `gorm:"type:LONGTEXT"` // Null when reposting
	InsertedAt time.Time      `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	CreatedAt  time.Time      `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	DeletedAt  sql.NullTime   `gorm:"type:DATETIME(6);default:NULL"`
	Privacy    string         `gorm:"type:ENUM('public','unlisted','follower','private');not null"` // enum treated as string
	ReplyToID  NullUUID       // Nullable fields as pointers
	RepostOfID NullUUID       // Nullable fields as pointers
	URI        sql.NullString `gorm:"size:512:default:NULL"`
	// PollID     NullUUID       // Nullable fields, assuming same type as ID

	Poster   User `gorm:"foreignKey:PosterID"`
	ReplyTo  *Post
	RepostOf *Post
	// Poll     *PostPoll
	Hashtags []PostHashtag `gorm:"foreignKey:PostID"`
	Mentions []PostMention `gorm:"foreignKey:PostID"`
}

type PostAttachment struct {
	ID             UUID
	PostID         UUID
	UploadedFileID UUID `gorm:"not null"`

	UploadedFile UploadedFile
	Post         Post
}

type UploadedFile struct {
	ID           UUID      `gorm:"primaryKey"`
	FileExt      string    `gorm:"size:128;not null"`
	CreatedAt    time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6);not null"`
	UploadedByID UUID      `gorm:"not null"`

	UploadedBy User
}

type PostFavorite struct {
	ID         uint64    `gorm:"primaryKey"`
	PostID     UUID      `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`
	UserID     UUID      `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`
	IsBookmark bool      `gorm:"uniqueIndex:idx_post_favorite_unique;not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	Post Post
	User User
}

type PostHashtag struct {
	ID          uint64 `gorm:"primaryKey"`
	PostID      UUID   `gorm:"not null"`
	HashtagName string `gorm:"size:255,index;not null"`

	Post Post
}

// type PostPoll struct {
// 	ID            UUID         `gorm:"primaryKey"`
// 	AllowMultiple bool         `gorm:"not null"`
// 	Due           sql.NullTime `gorm:"type:DATETIME(6)"`
// }

type PostReaction struct {
	ID         uint64    `gorm:"primaryKey"`
	PostID     UUID      `gorm:"not null"`
	ReactionID uint64    `gorm:"not null"`
	Reaction   Reaction  `gorm:"foreignKey:ReactionID"`
	UserID     UUID      `gorm:"not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	Post Post
	User User
}

type Reaction struct {
	ID   uint64 `gorm:"primaryKey"`
	Name string `gorm:"size:128;not null"`
}

// type PollChoice struct {
// 	ID     uint64 `gorm:"primaryKey"`
// 	PollID UUID   `gorm:"not null"`
// 	Title  string `gorm:"type:TEXT;not null"`
// 	Count  int64  `gorm:"default:0;not null"`

// 	Poll PostPoll
// }

// type PollVote struct {
// 	ID     uint64 `gorm:"primaryKey"`
// 	PollID UUID   `gorm:"not null"`
// 	UserID UUID   `gorm:"not null"`

// 	Poll PostPoll
// 	User User
// }

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

type UserFollowRequest struct {
	ID         UUID           `gorm:"primaryKey"`
	URI        sql.NullString `gorm:"size:512;uniqueIndex"`
	Incoming   bool           `gorm:"not null"`
	FollowerID UUID           `gorm:"not null"`
	FolloweeID UUID           `gorm:"not null"`
	CreatedAt  time.Time      `gorm:"autoCreateTime:nano;type:DATETIME(6)"`

	Follower User `gorm:"foreignKey:FollowerID"`
	Followee User `gorm:"foreignKey:FolloweeID"`
}

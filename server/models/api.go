package models

import (
	"time"

	"github.com/lightpub-dev/lightpub/db"
)

type PostRequest struct {
	Content  string  `json:"content"`
	Privacy  string  `json:"privacy" validate:"oneof=public unlisted follower private"`
	RepostOf *string `json:"repost_of_id,omitempty"`
	ReplyTo  *string `json:"reply_to_id,omitempty"`
}

type RepostRequest struct {
	Privacy string `json:"privacy" validate:"oneof=public unlisted follower private"`
}

type PostPollRequest struct {
	AllowMultiple bool       `json:"allow_multiple" validate:"required"`
	Due           *time.Time `json:"due" validate:"required"`
	Choices       []string   `json:"choices" validate:"required,min=2"`
}

type PostReactionRequest struct {
	Reaction string `json:"reaction" validate:"required"`
}

type UserPostListResponse struct {
	Posts []UserPostEntry `json:"results"`
}

type ReactionCountMap map[string]int64

type UserPostEntry struct {
	IDUUID    db.UUID             `json:"-"` // for internal use only
	ID        string              `json:"id"`
	Author    UserPostEntryAuthor `json:"author"`
	Content   *string             `json:"content"`
	CreatedAt time.Time           `json:"created_at"`
	Privacy   string              `json:"privacy"`

	ReplyTo  interface{} `json:"reply_to,omitempty"`  // string or UserPostEntry
	RepostOf interface{} `json:"repost_of,omitempty"` // string or UserPostEntry

	RepostCount   int64            `json:"repost_count"`
	FavoriteCount int64            `json:"favorite_count"`
	ReplyCount    int64            `json:"reply_count"`
	QuoteCount    int64            `json:"quote_count"`
	Reactions     ReactionCountMap `json:"reactions"`

	RepostedByMe   *bool `json:"reposted_by_me,omitempty"`
	FavoritedByMe  *bool `json:"favorited_by_me,omitempty"`
	BookmarkedByMe *bool `json:"bookmarked_by_me,omitempty"`
}

func (u *UserPostEntry) UpdateCounts(reply, favorite, repost, quote int64, reactions ReactionCountMap) {
	u.ReplyCount = reply
	u.FavoriteCount = favorite
	u.RepostCount = repost
	u.QuoteCount = quote
	u.Reactions = reactions
}

func (u *UserPostEntry) PostID() db.UUID {
	return u.IDUUID
}

type UserPostEntryAuthor struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Host     string `json:"host"`
	Nickname string `json:"nickname"`
}

type UserInfoResponse struct {
	ID          string                  `json:"id"`
	Username    string                  `json:"username"`
	Hostname    string                  `json:"hostname"`
	Nickname    string                  `json:"nickname"`
	URL         string                  `json:"url"`
	Bio         string                  `json:"bio"`
	IsFollowing *bool                   `json:"is_following,omitempty"`
	Counters    UserInfoCounterResponse `json:"counters"`
}

type UserInfoCounterResponse struct {
	Following int64 `json:"following"`
	Followers int64 `json:"followers"`
	Posts     int64 `json:"posts"`
}

type UserFullInfoResponse struct {
	UserInfoResponse
	Labels []UserLabel `json:"labels"`
}

type TimelineResponse struct {
	Posts          []UserPostEntry `json:"posts"`
	LatestPostTime *time.Time      `json:"latest_post_time"`
	OldestPostTime *time.Time      `json:"oldest_post_time"`
}

type UserProfileUpdate struct {
	Nickname *string     `json:"nickname" validate:"omitempty,max=128"`
	Bio      *string     `json:"bio" validate:"omitempty,max=2000"`
	Labels   []UserLabel `json:"labels" validate:"omitempty,max=4"`
}

type UserLabel struct {
	Key   string `json:"key" validate:"required,max=2000"`
	Value string `json:"value" validate:"required,max=2000"`
}

type TrendOverviewResponse struct {
	Trends []TrendResponse `json:"trends"`
}

type TrendResponse struct {
	Hashtag   string `json:"hashtag"`
	PostCount int64  `json:"post_count"`
}

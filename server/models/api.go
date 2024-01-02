package models

import "time"

type PostRequest struct {
	Content     string     `json:"content"`
	Privacy     string     `json:"privacy" validate:"oneof=public unlisted follower private"`
	ScheduledAt *time.Time `json:"scheduled_at"`

	Poll *PostPollRequest `json:"poll"`
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
	Posts []UserPostEntry `json:"posts"`
}

type UserPostEntry struct {
	ID        string              `json:"id"`
	Author    UserPostEntryAuthor `json:"author"`
	Content   string              `json:"content"`
	CreatedAt time.Time           `json:"created_at"`
	Privacy   string              `json:"privacy"`
}

type UserPostEntryAuthor struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Host     string `json:"host"`
}

type UserInfoResponse struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Hostname string `json:"hostname"`
	Nickname string `json:"nickname"`
	URL      string `json:"url"`
}

type TimelineResponse struct {
	Posts          []TimelinePostResponse `json:"posts"`
	LatestPostTime time.Time              `json:"latest_post_time"`
	OldestPostTime time.Time              `json:"oldest_post_time"`
}

type TimelinePostResponse struct {
	ID        string              `json:"id"`
	Author    UserPostEntryAuthor `json:"author"`
	Content   string              `json:"content"`
	CreatedAt time.Time           `json:"created_at"`
	Privacy   string              `json:"privacy"`
}

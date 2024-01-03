package timeline

import "time"

type FetchedPost struct {
	ID             string    `db:"id" json:"i"`
	PosterID       string    `db:"poster_id" json:"pi"`
	PosterUsername string    `db:"poster_username" json:"pu"`
	PosterHost     string    `db:"poster_host" json:"ph"`
	Content        *string   `db:"content" json:"c"`
	CreatedAt      time.Time `db:"created_at" json:"t"`
	Privacy        string    `db:"privacy" json:"pv"`

	ReplyTo  *string `db:"reply_to" json:"rt"`
	RepostOf *string `db:"repost_of" json:"rp"`
	PollID   *string `db:"poll_id" json:"pl"`
}

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

	RepostCount   int64 `db:"repost_count" json:"rc"`
	FavoriteCount int64 `db:"favorite_count" json:"fc"`
	ReplyCount    int64 `db:"reply_count" json:"r"`
	QuoteCount    int64 `db:"quote_count" json:"q"`
}

func (fp *FetchedPost) PostID() string {
	return fp.ID
}

func (fp *FetchedPost) UpdateCounts(reply, favorite, repost, quote int64) {
	fp.ReplyCount = reply
	fp.FavoriteCount = favorite
	fp.RepostCount = repost
	fp.QuoteCount = quote
}

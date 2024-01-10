package timeline

import (
	"time"

	"github.com/lightpub-dev/lightpub/models"
)

type FetchedPost struct {
	ID             string    `db:"id" json:"i"`
	PosterID       string    `db:"poster_id" json:"pi"`
	PosterUsername string    `db:"poster_username" json:"pu"`
	PosterHost     string    `db:"poster_host" json:"ph"`
	PosterNickname string    `db:"poster_nickname" json:"pn"`
	Content        *string   `db:"content" json:"c"`
	CreatedAt      time.Time `db:"created_at" json:"t"`
	Privacy        string    `db:"privacy" json:"pv"`

	ReplyTo  *string `db:"reply_to" json:"rt"`
	RepostOf *string `db:"repost_of" json:"rp"`
	PollID   *string `db:"poll_id" json:"pl"`

	RepostCount   int64                   `json:"rc"`
	FavoriteCount int64                   `json:"fc"`
	ReplyCount    int64                   `json:"r"`
	QuoteCount    int64                   `json:"q"`
	Reactions     models.ReactionCountMap `json:"rcs"`
}

func (fp *FetchedPost) PostID() string {
	return fp.ID
}

func (fp *FetchedPost) UpdateCounts(reply, favorite, repost, quote int64, reactions models.ReactionCountMap) {
	fp.ReplyCount = reply
	fp.FavoriteCount = favorite
	fp.RepostCount = repost
	fp.QuoteCount = quote
	fp.Reactions = reactions
}

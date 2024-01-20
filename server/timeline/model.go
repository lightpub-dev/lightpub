package timeline

import (
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

type FetchedPost struct {
	db.Post

	RepostCount   int64                   `json:"rc"`
	FavoriteCount int64                   `json:"fc"`
	ReplyCount    int64                   `json:"r"`
	QuoteCount    int64                   `json:"q"`
	Reactions     models.ReactionCountMap `json:"rcs"`

	RepostedByMe   *bool `db:"reposted_by_me" json:"rbm"`
	FavoritedByMe  *bool `db:"favorited_by_me" json:"fbm"`
	BookmarkedByMe *bool `db:"bookmarked_by_me" json:"bbm"`
}

func (fp *FetchedPost) PostID() db.UUID {
	return fp.ID
}

func (fp *FetchedPost) UpdateCounts(reply, favorite, repost, quote int64, reactions models.ReactionCountMap) {
	fp.ReplyCount = reply
	fp.FavoriteCount = favorite
	fp.RepostCount = repost
	fp.QuoteCount = quote
	fp.Reactions = reactions
}

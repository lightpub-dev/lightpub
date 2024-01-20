package timeline

import (
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

type FetchedPost struct {
	db.Post

	RepostCount   int64                   `json:"rc" gorm:"-"`
	FavoriteCount int64                   `json:"fc" gorm:"-"`
	ReplyCount    int64                   `json:"r" gorm:"-"`
	QuoteCount    int64                   `json:"q" gorm:"-"`
	Reactions     models.ReactionCountMap `json:"rcs" gorm:"-"`

	RepostedByMe   *bool `json:"rbm" gorm:"-"`
	FavoritedByMe  *bool `json:"fbm" gorm:"-"`
	BookmarkedByMe *bool `json:"bbm" gorm:"-"`
}

// FillInteractions implements posts.InteractionFillable.
func (fp *FetchedPost) FillInteractions(repostedByMe bool, favoritedByMe bool, bookmarkedByMe bool) {
	fp.RepostedByMe = &repostedByMe
	fp.FavoritedByMe = &favoritedByMe
	fp.BookmarkedByMe = &bookmarkedByMe
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

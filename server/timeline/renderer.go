package timeline

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
)

func FetchTimeline(ctx context.Context, conn db.DBConn, userID db.UUID, options FetchOptions) (*models.TimelineResponse, error) {
	// TODO: use timeline cache in redis
	// TODO: for now, just fetch from db
	cached, err := fetchPostsFromDB(ctx, conn, userID, options)
	if err != nil {
		return nil, err
	}

	// parse timeline
	timelinePosts := make([]models.UserPostEntry, 0, len(cached))
	oldestPost := time.Now()
	latestPost := time.Time{}
	for _, cache := range cached {
		var replyToURL, repostContent interface{}
		if cache.ReplyTo != nil {
			replyToURL = posts.CreatePostURL(*cache.ReplyToID)
		}
		if cache.RepostOf != nil {
			repost, err := posts.FetchSinglePostWithDepth(ctx, conn, *cache.RepostOfID, userID, 0)
			if err != nil {
				return nil, err
			}
			repostContent = repost
		}

		// TODO: Poll

		timelinePosts = append(timelinePosts, models.UserPostEntry{
			ID: cache.ID.String(),
			Author: models.UserPostEntryAuthor{
				ID:       cache.Poster.ID.String(),
				Username: cache.Poster.Username,
				Host:     cache.Poster.Host,
				Nickname: cache.Poster.Nickname,
			},
			Content:   cache.Content,
			CreatedAt: cache.CreatedAt,
			Privacy:   cache.Privacy,

			ReplyTo:  replyToURL,
			RepostOf: repostContent,

			ReplyCount:    cache.ReplyCount,
			RepostCount:   cache.RepostCount,
			FavoriteCount: cache.FavoriteCount,
			QuoteCount:    cache.QuoteCount,
			Reactions:     cache.Reactions,

			RepostedByMe:   cache.RepostedByMe,
			FavoritedByMe:  cache.FavoritedByMe,
			BookmarkedByMe: cache.BookmarkedByMe,
		})

		if cache.CreatedAt.Before(oldestPost) {
			oldestPost = cache.CreatedAt
		}
		if cache.CreatedAt.After(latestPost) {
			latestPost = cache.CreatedAt
		}
	}

	var oldestPostPtr, latestPostPtr *time.Time
	if len(cached) != 0 {
		oldestPostPtr = &oldestPost
		latestPostPtr = &latestPost
	}

	return &models.TimelineResponse{
		Posts:          timelinePosts,
		OldestPostTime: oldestPostPtr,
		LatestPostTime: latestPostPtr,
	}, nil
}

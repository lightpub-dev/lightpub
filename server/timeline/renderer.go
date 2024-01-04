package timeline

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
)

func FetchTimeline(ctx context.Context, conn db.DBConn, userID string, options FetchOptions) (*models.TimelineResponse, error) {
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
		var replyToURL, repostOfURL interface{}
		if cache.ReplyTo != nil {
			replyToURL = posts.CreatePostURL(*cache.ReplyTo)
		}
		if cache.RepostOf != nil {
			repostOfURL = posts.CreatePostURL(*cache.RepostOf)
		}

		// TODO: Poll

		timelinePosts = append(timelinePosts, models.UserPostEntry{
			ID: cache.ID,
			Author: models.UserPostEntryAuthor{
				ID:       cache.PosterID,
				Username: cache.PosterUsername,
				Host:     cache.PosterHost,
			},
			Content:   cache.Content,
			CreatedAt: cache.CreatedAt,
			Privacy:   cache.Privacy,

			ReplyTo:  replyToURL,
			RepostOf: repostOfURL,

			ReplyCount:    cache.ReplyCount,
			RepostCount:   cache.RepostCount,
			FavoriteCount: cache.FavoriteCount,
			QuoteCount:    cache.QuoteCount,
			Reactions:     cache.Reactions,
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

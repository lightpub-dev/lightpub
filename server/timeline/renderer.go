package timeline

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/redis/go-redis/v9"
)

func FetchTimeline(ctx context.Context, tx db.DBOrTx, rdb *redis.Client, userID string, options FetchOptions) (*models.TimelineResponse, error) {
	// TODO: use timeline cache in redis
	// TODO: for now, just fetch from db
	cached, err := fetchPostsFromDB(ctx, tx, userID, options)
	if err != nil {
		return nil, err
	}

	// parse timeline
	posts := make([]models.TimelinePostResponse, 0, len(cached))
	oldestPost := time.Now()
	latestPost := time.Time{}
	for _, cache := range cached {
		posts = append(posts, models.TimelinePostResponse{
			ID: cache.ID,
			Author: models.UserPostEntryAuthor{
				ID:       cache.PosterID,
				Username: cache.PosterUsername,
				Host:     cache.PosterHost,
			},
			Content:   cache.Content,
			CreatedAt: cache.CreatedAt,
			Privacy:   cache.Privacy,
		})

		if cache.CreatedAt.Before(oldestPost) {
			oldestPost = cache.CreatedAt
		}
		if cache.CreatedAt.After(latestPost) {
			latestPost = cache.CreatedAt
		}
	}

	return &models.TimelineResponse{
		Posts:          posts,
		OldestPostTime: oldestPost,
		LatestPostTime: latestPost,
	}, nil
}

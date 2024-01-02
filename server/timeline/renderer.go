package timeline

import (
	"context"
	"encoding/json"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/redis/go-redis/v9"
)

func FetchTimeline(ctx context.Context, tx db.DBOrTx, rdb *redis.Client, userID string) (*models.TimelineResponse, error) {
	rkey := timelineRedisKey(userID)

	// check if timeline is cached
	exists, err := rdb.Exists(ctx, rkey).Result()
	if err != nil {
		return nil, err
	}

	if exists == 0 {
		// rebuild timeline
		if err = rebuildTimeline(ctx, tx, rdb, userID); err != nil {
			return nil, err
		}
	}

	// fetch timeline
	cached, err := rdb.LRange(ctx, rkey, 0, -1).Result()
	if err != nil {
		return nil, err
	}

	// parse timeline
	posts := make([]models.TimelinePostResponse, 0, len(cached))
	oldestPost := time.Now()
	latestPost := time.Time{}
	for _, cacheStr := range cached {
		var cache FetchedPost
		if err = json.Unmarshal([]byte(cacheStr), &cache); err != nil {
			return nil, err
		}
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

package timeline

import (
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/utils"
)

func (s *DBTimelineService) FetchTimeline(userID db.UUID, options FetchOptions) (*models.TimelineResponse, error) {
	// TODO: use timeline cache in redis
	// TODO: for now, just fetch from db
	cached, err := s.fetchPostsFromDB(userID, options)
	if err != nil {
		return nil, err
	}

	// parse timeline
	timelinePosts := make([]models.UserPostEntry, 0, len(cached))
	oldestPost := time.Now()
	latestPost := time.Time{}
	for _, cache := range cached {
		var replyToURL, repostContent interface{}
		if cache.ReplyToID.Valid {
			replyToURL = posts.CreatePostURL(cache.ReplyToID.UUID)
		}
		if cache.RepostOfID.Valid {
			repost, err := s.postFetch.FetchSinglePostWithDepth(cache.RepostOfID.UUID, userID, 0)
			if err != nil {
				return nil, err
			}
			repostContent = repost
		}

		// TODO: Poll

		timelinePosts = append(timelinePosts, models.UserPostEntry{
			ID:       cache.ID,
			IDString: cache.ID.String(),
			Author: models.UserPostEntryAuthor{
				ID:       cache.Poster.ID.String(),
				Username: cache.Poster.Username,
				Host:     utils.ConvertSqlHost(cache.Poster.Host),
				Nickname: cache.Poster.Nickname,
			},
			Content:   utils.ConvertSqlStringToPtr(cache.Content),
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

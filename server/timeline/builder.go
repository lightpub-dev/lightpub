package timeline

import (
	"context"
	"sort"
	"time"

	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	pts "github.com/lightpub-dev/lightpub/posts"
)

const (
	DefaultTimelineSize = 20
)

func timelineRedisKey(userID string) string {
	return "timeline:" + userID
}

type FetchOptions struct {
	BeforeTime *time.Time
	AfterTime  *time.Time
	Limit      int
}

func fetchPostsFromDB(ctx context.Context, conn db.DBConn, userID db.UUID, options FetchOptions) ([]FetchedPost, error) {
	// limit := DefaultTimelineSize
	// if options.Limit > 0 {
	// 	limit = options.Limit
	// }

	// retrieve my latest posts
	var posts []FetchedPost
	myPostsQuery := conn.DB().Model(&db.Post{}).Joins("Poster").Where("poster_id = ?", userID).Order("created_at DESC").Limit(options.Limit)
	if options.BeforeTime != nil {
		myPostsQuery = myPostsQuery.Where("created_at < ?", options.BeforeTime)
	}
	if options.AfterTime != nil {
		myPostsQuery = myPostsQuery.Where("created_at > ?", options.AfterTime)
	}
	if err := myPostsQuery.Find(&posts).Error; err != nil {
		return nil, err
	}

	// retrieve my following's latest posts
	var followingPosts []FetchedPost
	followingQuery := conn.DB().Model(&db.Post{}).Joins("Poster").Joins("INNER JOIN user_follows pf ON pf.followee_id=Poster.id").Where("pf.follower_id = ?", userID).Where("privacy IN ('public','follower')").Order("created_at DESC").Limit(options.Limit)
	if options.BeforeTime != nil {
		followingQuery = followingQuery.Where("created_at < ?", options.BeforeTime)
	}
	if options.AfterTime != nil {
		followingQuery = followingQuery.Where("created_at > ?", options.AfterTime)
	}
	if err := followingQuery.Find(&followingPosts).Error; err != nil {
		return nil, err
	}

	// retrieve latest posts which mention me
	var mentionPosts []FetchedPost
	mentionQuery := conn.DB().Model(&db.Post{}).Joins("Poster").Joins("Mentions").Where("Mentions.target_user_id = ?", userID).Order("created_at DESC").Limit(options.Limit)
	if options.BeforeTime != nil {
		mentionQuery = mentionQuery.Where("created_at < ?", options.BeforeTime)
	}
	if options.AfterTime != nil {
		mentionQuery = mentionQuery.Where("created_at > ?", options.AfterTime)
	}
	if err := mentionQuery.Find(&mentionPosts).Error; err != nil {
		return nil, err
	}

	// merge these posts
	posts = append(posts, followingPosts...)
	posts = append(posts, mentionPosts...)

	// sort by created_at DESC
	sort.Slice(posts, func(i, j int) bool {
		return posts[i].CreatedAt.After(posts[j].CreatedAt)
	})

	// add additional info to each post
	for i := range posts {
		// Add hostname if empty
		if posts[i].Poster.Host == "" {
			posts[i].Poster.Host = config.MyHostname
		}

		// Fill in count fields
		if err := pts.FillCounts(ctx, conn, &posts[i]); err != nil {
			return nil, err
		}

		// Fill in interactions
		if err := pts.FillInteraction(conn, userID, &posts[i]); err != nil {
			return nil, err
		}
	}

	return posts, nil
}

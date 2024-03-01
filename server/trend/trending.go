package trend

import (
	"context"
	"sort"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/utils"
)

type trendList struct {
	HashtagName string
	PostCount   int64
}

// GetCurrentTrend fetches the most posted hashtags within the last 3 hours
func GetCurrentTrend(ctx context.Context, conn db.DBConn) (*models.TrendOverviewResponse, error) {
	result := []trendList{}
	hashtagAndCounts := conn.DB().Model(&db.PostHashtag{}).InnerJoins("Post").Where("Post.created_at >= DATE_SUB(NOW(), INTERVAL 24 HOUR) AND Post.privacy='public'").Select("post_hashtags.hashtag_name, Post.id AS post_id")
	err := conn.DB().Table("(?) AS hc", hashtagAndCounts).Select("hc.hashtag_name, COUNT(hc.post_id) AS post_count").Group("hashtag_name").Order("post_count DESC").Limit(5).Find(&result).Error
	if err != nil {
		return nil, err
	}

	// transform into models.TrendOverviewResponse
	var trends []models.TrendResponse
	for _, trend := range result {
		trends = append(trends, models.TrendResponse{
			Hashtag:   trend.HashtagName,
			PostCount: trend.PostCount,
		})
	}

	return &models.TrendOverviewResponse{
		Trends: trends,
	}, nil
}

// GetTrendPosts fetches the posts for a given hashtag
// Consider viewerID to determine if the posts are visible to the viewer
func GetTrendPosts(ctx context.Context, conn db.DBConn, hashtag string, viewerID db.UUID, beforeDate *time.Time, limit int) ([]models.UserPostEntry, error) {
	// get 'public' posts
	var publicPosts []db.Post
	publicQuery := conn.DB().Model(&db.Post{}).Joins("Poster").Joins("Hashtags").Where("Hashtags.hashtag_name = ? AND posts.privacy = 'public'", hashtag).Order("posts.created_at DESC").Limit(limit)
	if beforeDate != nil {
		publicQuery = publicQuery.Where("posts.created_at < ?", *beforeDate)
	}
	err := publicQuery.Find(&publicPosts).Error
	if err != nil {
		return nil, err
	}

	// get 'follower' posts
	var followerPosts []db.Post
	if viewerID != (db.UUID{}) {
		err := conn.DB().Model(&db.Post{}).Joins("Poster").Joins("Hashtags").Joins("INNER JOIN user_follows uf ON uf.followee_id=Poster.id").Where("posts.privacy = 'follower'").Where("Hashtags.hashtag_name = ?", hashtag).Where("uf.follower_id = ?", viewerID).Order("posts.created_at DESC").Limit(limit).Find(&followerPosts).Error
		if err != nil {
			return nil, err
		}
	}

	// get 'private' posts
	var privatePosts []db.Post
	if viewerID != (db.UUID{}) {
		// TODO: implement
	}

	// merge all rawPosts
	rawPosts := append(publicPosts, followerPosts...)
	rawPosts = append(rawPosts, privatePosts...)

	// sort by created_at DESC
	sort.Slice(rawPosts, func(i, j int) bool {
		return rawPosts[i].CreatedAt.After(rawPosts[j].CreatedAt)
	})

	// limit
	if len(rawPosts) > limit {
		rawPosts = rawPosts[:limit]
	}

	var result []models.UserPostEntry = []models.UserPostEntry{}
	for _, post := range rawPosts {
		entry := (models.UserPostEntry{
			ID: post.ID.String(),
			Author: models.UserPostEntryAuthor{
				ID:       post.Poster.ID.String(),
				Username: post.Poster.Username,
				Host:     utils.ConvertSqlHost(post.Poster.Host),
				Nickname: post.Poster.Nickname,
			},
			Content:   utils.ConvertSqlStringToPtr(post.Content),
			CreatedAt: post.CreatedAt,
			Privacy:   post.Privacy,
			ReplyTo:   post.ReplyTo,
			RepostOf:  post.RepostOf,
		})
		posts.FillCounts(ctx, conn, &entry)

		result = append(result, entry)
	}

	return result, nil
}

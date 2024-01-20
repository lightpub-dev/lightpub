package trend

import (
	"context"
	"sort"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
)

type trendList struct {
	HashtagName string `db:"hashtag_name"`
	PostCount   int64  `db:"post_count"`
}

// GetCurrentTrend fetches the most posted hashtags within the last 3 hours
func GetCurrentTrend(ctx context.Context, conn db.DBConn) (*models.TrendOverviewResponse, error) {
	result := []trendList{}
	err := conn.DB().Model(&db.PostHashtag{}).Joins("Post p").Where("p.created_at >= DATE_SUB(NOW(), INTERVAL 24 HOUR) AND p.privacy='public'").Group("post_hashtags.hashtag_name").Order("post_count DESC").Limit(5).Select("post_hashtags.hashtag_name, COUNT(p.id) AS post_count").Scan(&result).Error
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
	publicQuery := conn.DB().Model(&db.Post{}).Joins("Poster").Joins("Hashtags").Where("post_hashtags.hashtag_name = ? AND posts.privacy = 'public'", hashtag).Order("posts.created_at DESC").Limit(limit)
	if beforeDate != nil {
		publicQuery = publicQuery.Where("posts.created_at < ?", beforeDate)
	}
	err := publicQuery.Find(&publicPosts).Error
	if err != nil {
		return nil, err
	}

	// get 'follower' posts
	var followerPosts []db.Post
	if viewerID != (db.UUID{}) {
		// TODO:
		// 		sql = `
		// SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,u.nickname AS poster_nickname,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
		// FROM Post p
		// INNER JOIN User u ON p.poster_id=u.id
		// INNER JOIN PostHashtag ph ON ph.post_id=p.id
		// INNER JOIN UserFollow uf ON p.poster_id=uf.followee_id
		// WHERE
		//   ph.hashtag_name=?
		//   AND p.scheduled_at IS NULL
		//   AND p.privacy='follower'
		//   AND uf.follower_id=UUID_TO_BIN(?)
		//   `
		// 		params = []interface{}{hashtag, viewerID}
		// 		if beforeDate != nil {
		// 			sql += " AND p.created_at < ?"
		// 			params = append(params, beforeDate)
		// 		}
		// 		sql += " ORDER BY p.created_at DESC LIMIT ?"
		// 		params = append(params, limit)

		// 		err = conn.DB().SelectContext(ctx, &followerPosts, sql, params...)
		// 		if err != nil {
		// 			return nil, err
		// 		}
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

	var result []models.UserPostEntry
	for _, post := range rawPosts {
		entry := (models.UserPostEntry{
			ID: post.ID.String(),
			Author: models.UserPostEntryAuthor{
				ID:       post.Poster.ID.String(),
				Username: post.Poster.Username,
				Host:     post.Poster.Host,
				Nickname: post.Poster.Nickname,
			},
			Content:   post.Content,
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

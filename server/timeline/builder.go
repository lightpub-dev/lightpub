package timeline

import (
	"sort"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	pts "github.com/lightpub-dev/lightpub/posts"
)

const (
	DefaultTimelineSize = 20
)

type TimelineService interface {
	FetchTimeline(userID db.UUID, options FetchOptions) (*models.TimelineResponse, error)
}

type DBTimelineService struct {
	conn            db.DBConn
	postInteraction pts.PostInteractionService
	postCount       pts.PostCountService
	postFetch       pts.PostFetchService
}

func ProvideDBTimelineService(conn db.DBConn, postInteraction pts.PostInteractionService, postCount pts.PostCountService, postFetch pts.PostFetchService) *DBTimelineService {
	return &DBTimelineService{conn, postInteraction, postCount, postFetch}
}

func timelineRedisKey(userID string) string {
	return "timeline:" + userID
}

type FetchOptions struct {
	BeforeTime *time.Time
	AfterTime  *time.Time
	Limit      int
}

func (s *DBTimelineService) fetchPostsFromDB(userID db.UUID, options FetchOptions) ([]FetchedPost, error) {
	// limit := DefaultTimelineSize
	// if options.Limit > 0 {
	// 	limit = options.Limit
	// }
	conn := s.conn.DB

	// retrieve my latest posts
	var posts []FetchedPost
	myPostsQuery := conn.Model(&db.Post{}).Joins("Poster").Where("poster_id = ?", userID).Order("created_at DESC").Limit(options.Limit)
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
	followingQuery := conn.Model(&db.Post{}).Joins("Poster").Joins("INNER JOIN user_follows pf ON pf.followee_id=Poster.id").Where("pf.follower_id = ?", userID).Where("privacy IN ('public','follower')").Order("created_at DESC").Limit(options.Limit)
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
	mentionQuery := conn.Model(&db.Post{}).Joins("Poster").Joins("Mentions").Where("Mentions.target_user_id = ?", userID).Order("created_at DESC").Limit(options.Limit)
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
		// Fill in count fields
		if err := s.postCount.FillCounts(&posts[i]); err != nil {
			return nil, err
		}

		// Fill in interactions
		if err := s.postInteraction.FillInteraction(userID, &posts[i]); err != nil {
			return nil, err
		}
	}

	return posts, nil
}

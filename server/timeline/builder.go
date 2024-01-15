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

func fetchPostsFromDB(ctx context.Context, conn db.DBConn, userID string, options FetchOptions) ([]FetchedPost, error) {
	limit := DefaultTimelineSize
	if options.Limit > 0 {
		limit = options.Limit
	}

	// retrieve my latest posts
	var posts []FetchedPost
	mySql := `
	SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,u.nickname AS poster_nickname,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me
	FROM Post p
	INNER JOIN User u ON p.poster_id=u.id
	WHERE
		p.poster_id=UUID_TO_BIN(?)
		AND p.scheduled_at IS NULL
	`
	myParams := []interface{}{userID, userID, userID, userID}
	if options.BeforeTime != nil {
		mySql += ` AND p.created_at < ?`
		myParams = append(myParams, options.BeforeTime)
	}
	if options.AfterTime != nil {
		mySql += ` AND p.created_at > ?`
		myParams = append(myParams, options.AfterTime)
	}
	mySql += ` ORDER BY p.created_at DESC LIMIT ?`
	myParams = append(myParams, limit)

	err := conn.DB().SelectContext(ctx, &posts, mySql, myParams...)
	if err != nil {
		return nil, err
	}

	// retrieve my following's latest posts
	var followingPosts []FetchedPost
	followingSql := `
	SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,u.nickname AS poster_nickname,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me
	FROM Post p
	INNER JOIN User u ON p.poster_id=u.id
	INNER JOIN UserFollow uf ON p.poster_id=uf.followee_id
	WHERE
		uf.follower_id=UUID_TO_BIN(?)
		AND p.privacy IN ('public','follower')
		AND p.scheduled_at IS NULL
	`
	followingParams := []interface{}{userID, userID, userID, userID}
	if options.BeforeTime != nil {
		followingSql += ` AND p.created_at < ?`
		followingParams = append(followingParams, options.BeforeTime)
	}
	if options.AfterTime != nil {
		followingSql += ` AND p.created_at > ?`
		followingParams = append(followingParams, options.AfterTime)
	}
	followingSql += ` ORDER BY p.created_at DESC LIMIT ?`
	followingParams = append(followingParams, limit)

	err = conn.DB().SelectContext(ctx, &followingPosts, followingSql, followingParams...)
	if err != nil {
		return nil, err
	}

	// retrieve latest posts which mention me
	var mentionPosts []FetchedPost
	mentionSql := `
	SELECT BIN_TO_UUID(p.id) AS id,BIN_TO_UUID(p.poster_id) AS poster_id,u.username AS poster_username,u.host AS poster_host,u.nickname AS poster_nickname,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me
	FROM Post p
	INNER JOIN User u ON p.poster_id=u.id
	INNER JOIN PostMention pm ON p.id=pm.post_id
	WHERE
		pm.target_user_id=UUID_TO_BIN(?)
		AND p.scheduled_at IS NULL
	`
	mentionParams := []interface{}{userID, userID, userID, userID}
	if options.BeforeTime != nil {
		mentionSql += ` AND p.created_at < ?`
		mentionParams = append(mentionParams, options.BeforeTime)
	}
	if options.AfterTime != nil {
		mentionSql += ` AND p.created_at > ?`
		mentionParams = append(mentionParams, options.AfterTime)
	}
	mentionSql += ` ORDER BY p.created_at DESC LIMIT ?`
	mentionParams = append(mentionParams, limit)

	err = conn.DB().SelectContext(ctx, &mentionPosts, mentionSql, mentionParams...)
	if err != nil {
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
		if posts[i].PosterHost == "" {
			posts[i].PosterHost = config.MyHostname
		}

		// Fill in count fields
		if err := pts.FillCounts(ctx, conn, &posts[i]); err != nil {
			return nil, err
		}
	}

	return posts, nil
}

// func rebuildTimeline(dbio *db.DBIO, userID string) error {
// 	posts, err := fetchPostsFromDB(dbio, userID, FetchOptions{})
// 	if err != nil {
// 		return err
// 	}

// 	// reset timeline
// 	pipe := rdb.Pipeline()
// 	rkey := timelineRedisKey(userID)
// 	_, err = pipe.Del(dbio.Ctx, rkey).Result()
// 	if err != nil {
// 		return err
// 	}

// 	// add posts to timeline
// 	// newer left <---> right older
// 	for _, post := range posts {
// 		postJson, err := json.Marshal(post)
// 		if err != nil {
// 			return err
// 		}
// 		if _, err = pipe.RPush(ctx, rkey, postJson).Result(); err != nil {
// 			return err
// 		}
// 	}

// 	// exec
// 	_, err = pipe.Exec(ctx)
// 	return err
// }

// func AddToTimeline(ctx context.Context, tx db.DBOrTx, rdb *redis.Client, targetUserID string, post FetchedPost) error {
// 	// add post to timeline
// 	postJson, err := json.Marshal(post)
// 	if err != nil {
// 		return err
// 	}
// 	rkey := timelineRedisKey(targetUserID)

// 	// check existence
// 	exists, err := rdb.Exists(ctx, rkey).Result()
// 	if err != nil {
// 		return err
// 	}

// 	// if not exists, rebuild timeline
// 	if exists == 0 {
// 		if err = rebuildTimeline(ctx, tx, rdb, targetUserID); err != nil {
// 			return err
// 		}
// 	}

// 	_, err = rdb.LPush(ctx, rkey, postJson).Result()
// 	if err != nil {
// 		return err
// 	}

// 	return nil
// }

// func ClearTimeline(ctx context.Context, rdb *redis.Client, userID string) error {
// 	rkey := timelineRedisKey(userID)
// 	_, err := rdb.Del(ctx, rkey).Result()
// 	return err
// }

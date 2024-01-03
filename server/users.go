package main

import (
	"fmt"
	"net/http"
	"sort"
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/users"
)

const (
	DefaultUserPostLimit   = 10
	DefaultFollowViewLimit = 10
)

func getUserPosts(c echo.Context) error {
	authed := c.Get(ContextAuthed).(bool)
	var viewerUserID string
	if authed {
		viewerUserID = c.Get(ContextUserID).(string)
	}
	username := c.Param("username")

	targetUser, err := users.FindIDByUsername(c.Request().Context(), db, username)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(404, "user not found")
	}

	limitStr := c.QueryParam("limit")
	var limit int
	if limitStr == "" {
		limit = DefaultUserPostLimit
	} else {
		limit64, err := strconv.ParseInt(limitStr, 10, 32)
		if err != nil {
			return c.String(400, "invalid limit")
		}
		if limit64 < 0 {
			return c.String(400, "invalid limit")
		}
		limit = int(limit64)
	}

	// first, get all "public" and "unlisted" posts
	var publicPosts []models.Post
	err = db.Select(&publicPosts, `
	SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
		FROM Post p
	WHERE
		p.poster_id=UUID_TO_BIN(?)
		AND p.privacy IN ('public','unlisted')
		AND p.scheduled_at IS NULL
	ORDER BY p.created_at DESC
	LIMIT ?
	`, targetUser.ID, limit)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	// "follower" posts...
	var followerPosts []models.Post
	if viewerUserID != "" {
		isFollowed := false
		if viewerUserID == targetUser.ID {
			// when viewer is target itself...
			isFollowed = true
		}
		if !isFollowed {
			// check if user is followed by target
			isFollowed, err = users.IsFollowedBy(c.Request().Context(), db, viewerUserID, targetUser.ID)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}
		if isFollowed {
			// fetch "follower" posts
			err = db.Select(&followerPosts, `
		SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
		FROM Post p
		WHERE
			p.poster_id=UUID_TO_BIN(?)
			AND p.privacy = 'follower'
			AND p.scheduled_at IS NULL
		ORDER BY p.created_at DESC
		LIMIT ?
		`, targetUser.ID, limit)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}
	}

	// fetch "private" posts
	var privatePosts []models.Post
	if viewerUserID != "" {
		if targetUser.ID == viewerUserID {
			// when viewer is target itself, fetch all private posts
			err = db.Select(&privatePosts, `
			SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
		FROM Post p
		WHERE
			p.poster_id=UUID_TO_BIN(?)
			AND p.privacy = 'private'
			AND p.scheduled_at IS NULL
		ORDER BY p.created_at DESC
		LIMIT ?
		`, targetUser.ID, limit)
		} else {
			err = db.Select(&privatePosts, `
	SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id
	FROM Post p
	INNER JOIN PostMention pm ON p.id=pm.post_id
	WHERE
		p.poster_id=UUID_TO_BIN(?)
		AND p.privacy = 'private'
		AND p.scheduled_at IS NULL
		AND pm.target_user_id=UUID_TO_BIN(?)
	ORDER BY p.created_at DESC
	LIMIT ?
	`, targetUser.ID, viewerUserID, limit)
		}
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	}

	// merge these allPosts
	allPosts := append(publicPosts, followerPosts...)
	allPosts = append(allPosts, privatePosts...)
	// sort by created_at desc
	sort.Slice(allPosts, func(i int, j int) bool {
		createdAtI := allPosts[i].CreatedAt
		createdAtJ := allPosts[j].CreatedAt
		return createdAtI.After(createdAtJ) // DESC
	})
	// limit to limit
	if len(allPosts) > limit {
		allPosts = allPosts[:limit]
	}

	// convert to response
	resp := []models.UserPostEntry{}
	for _, post := range allPosts {
		hostname := targetUser.Host
		if hostname == "" {
			hostname = config.MyHostname
		}

		var replyToPostOrURL, repostOfPostOrURL interface{}
		if post.ReplyTo != nil {
			replyToPostOrURL = posts.CreatePostURL(*post.ReplyTo)
		}
		if post.RepostOf != nil {
			repostOfPostOrURL = posts.CreatePostURL(*post.RepostOf)
		}

		resp = append(resp, models.UserPostEntry{
			ID: post.ID,
			Author: models.UserPostEntryAuthor{
				ID:       targetUser.ID,
				Username: targetUser.Username,
				Host:     hostname,
			},
			Content:   post.Content,
			CreatedAt: post.CreatedAt,
			Privacy:   post.Privacy,

			ReplyTo:  replyToPostOrURL,
			RepostOf: repostOfPostOrURL,
			// TODO: Poll
		})
	}

	return c.JSON(http.StatusOK,
		models.UserPostListResponse{
			Posts: resp,
		})
}

func createLocalUserURL(username string) string {
	return fmt.Sprintf("%s/user/%s", config.BaseURL, username)
}

func getUserFollowerOrFollowing(c echo.Context, fetchFollower bool) error {
	username := c.Param("username")

	limitStr := c.QueryParam("limit")
	limit := DefaultFollowViewLimit
	if limitStr != "" {
		limit64, err := strconv.ParseInt(limitStr, 10, 32)
		if err != nil {
			return c.String(400, "invalid limit")
		}
		if limit64 < 0 {
			return c.String(400, "invalid limit")
		}
		limit = int(limit64)
	}
	afterDateStr := c.QueryParam("after_date")
	var afterDate *time.Time
	if afterDateStr != "" {
		afterDateParsed, err := time.Parse(time.RFC3339, afterDateStr)
		if err != nil {
			return c.String(400, "invalid after_date")
		}
		afterDate = &afterDateParsed
	}

	targetUser, err := users.FindIDByUsername(c.Request().Context(), db, username)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(404, "user not found")
	}

	var sql string
	if fetchFollower {
		sql += `
	SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,u.nickname,u.url
	FROM User u
	INNER JOIN UserFollow uf ON u.id=uf.follower_id
	WHERE uf.followee_id=UUID_TO_BIN(?)`
	} else {
		sql += `
	SELECT BIN_TO_UUID(u.id) AS id,u.username,u.host,u.nickname,u.url
	FROM User u
	INNER JOIN UserFollow uf ON u.id=uf.followee_id
	WHERE uf.follower_id=UUID_TO_BIN(?)`
	}
	if afterDate != nil {
		sql += ` AND uf.created_at > ?`
	}
	sql += ` ORDER BY uf.created_at DESC`
	sql += ` LIMIT ?`

	var followDB []models.User
	err = db.Select(&followDB, sql, targetUser.ID, limit)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	follows := []models.UserInfoResponse{}
	for _, follow := range followDB {
		hostname := config.MyHostname
		if follow.Host != "" {
			hostname = follow.Host
		}
		var userURL string
		if follow.URL == nil {
			userURL = createLocalUserURL(follow.Username)
		} else {
			userURL = *follow.URL
		}
		follows = append(follows, models.UserInfoResponse{
			ID:       follow.ID,
			Username: follow.Username,
			Hostname: hostname,
			Nickname: follow.Nickname,
			URL:      userURL,
		})
	}

	var jsonKey string
	if fetchFollower {
		jsonKey = "followers"
	} else {
		jsonKey = "followings"
	}
	return c.JSON(http.StatusOK, map[string]interface{}{
		jsonKey: follows,
	})
}

func getUserFollowers(c echo.Context) error {
	return getUserFollowerOrFollowing(c, true)
}

func getUserFollowing(c echo.Context) error {
	return getUserFollowerOrFollowing(c, false)
}

func modifyFollow(c echo.Context, isFollow bool) error {
	myUserId := c.Get(ContextUserID).(string)
	targetUsername := c.Param("username")

	tx, err := db.Beginx()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	defer tx.Rollback()

	// existence check
	targetUser, err := users.FindIDByUsername(c.Request().Context(), tx, targetUsername)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	if isFollow {
		_, err := tx.Exec("INSERT INTO UserFollow (follower_id, followee_id) VALUES (UUID_TO_BIN(?), UUID_TO_BIN(?)) ON DUPLICATE KEY UPDATE id=id", myUserId, targetUser.ID)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	} else {
		_, err := tx.Exec("DELETE FROM UserFollow WHERE follower_id=UUID_TO_BIN(?) AND followee_id=UUID_TO_BIN(?)", myUserId, targetUser.ID)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	}

	err = tx.Commit()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	return c.NoContent(http.StatusOK)
}

func followAUser(c echo.Context) error {
	return modifyFollow(c, true)
}

func unfollowAUser(c echo.Context) error {
	return modifyFollow(c, false)
}

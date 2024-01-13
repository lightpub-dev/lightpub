package api

import (
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

type postWithRepostedByMe struct {
	models.Post
	RepostedByMe *bool `db:"reposted_by_me"`
}

func (h *Handler) GetUserPosts(c echo.Context) error {
	authed := c.Get(ContextAuthed).(bool)
	var viewerUserID string
	if authed {
		viewerUserID = c.Get(ContextUserID).(string)
	}
	username := c.Param("username")

	targetUser, err := users.FindIDByUsername(c.Request().Context(), h.MakeDB(), username)
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
	var publicPosts []postWithRepostedByMe
	err = h.DB.Select(&publicPosts, `
	SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(?))) AS reposted_by_me
		FROM Post p
	WHERE
		p.poster_id=UUID_TO_BIN(?)
		AND p.privacy IN ('public','unlisted')
		AND p.scheduled_at IS NULL
	ORDER BY p.created_at DESC
	LIMIT ?
	`, viewerUserID, viewerUserID, targetUser.ID, limit)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	// "follower" posts...
	var followerPosts []postWithRepostedByMe
	if viewerUserID != "" {
		isFollowed := false
		if viewerUserID == targetUser.ID {
			// when viewer is target itself...
			isFollowed = true
		}
		if !isFollowed {
			// check if user is followed by target
			isFollowed, err = users.IsFollowedBy(c.Request().Context(), h.MakeDB(), viewerUserID, targetUser.ID)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}
		if isFollowed {
			// fetch "follower" posts
			err = h.DB.Select(&followerPosts, `
		SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
		IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(?))) AS reposted_by_me
		FROM Post p
		WHERE
			p.poster_id=UUID_TO_BIN(?)
			AND p.privacy = 'follower'
			AND p.scheduled_at IS NULL
		ORDER BY p.created_at DESC
		LIMIT ?
		`, viewerUserID, viewerUserID, targetUser.ID, limit)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}
	}

	// fetch "private" posts
	var privatePosts []postWithRepostedByMe
	if viewerUserID != "" {
		if targetUser.ID == viewerUserID {
			// when viewer is target itself, fetch all private posts
			err = h.DB.Select(&privatePosts, `
			SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
			IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(?))) AS reposted_by_me
		FROM Post p
		WHERE
			p.poster_id=UUID_TO_BIN(?)
			AND p.privacy = 'private'
			AND p.scheduled_at IS NULL
		ORDER BY p.created_at DESC
		LIMIT ?
		`, viewerUserID, viewerUserID, targetUser.ID, limit)
		} else {
			err = h.DB.Select(&privatePosts, `
	SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(?))) AS reposted_by_me
	FROM Post p
	INNER JOIN PostMention pm ON p.id=pm.post_id
	WHERE
		p.poster_id=UUID_TO_BIN(?)
		AND p.privacy = 'private'
		AND p.scheduled_at IS NULL
		AND pm.target_user_id=UUID_TO_BIN(?)
	ORDER BY p.created_at DESC
	LIMIT ?
	`, viewerUserID, viewerUserID, targetUser.ID, viewerUserID, limit)
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
			repostOfPostOrURL, err = posts.FetchSinglePostWithDepth(c.Request().Context(), h.MakeDB(), *post.RepostOf, "", 0)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}

		resp = append(resp, models.UserPostEntry{
			ID: post.ID,
			Author: models.UserPostEntryAuthor{
				ID:       targetUser.ID,
				Username: targetUser.Username,
				Host:     hostname,
				Nickname: targetUser.Nickname,
			},
			Content:   post.Content,
			CreatedAt: post.CreatedAt,
			Privacy:   post.Privacy,

			ReplyTo:  replyToPostOrURL,
			RepostOf: repostOfPostOrURL,
			// TODO: Poll

			RepostedByMe: post.RepostedByMe,
		})
	}

	for i := range resp {
		// add counts
		if err := posts.FillCounts(c.Request().Context(), h.MakeDB(), &resp[i]); err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	}

	return c.JSON(http.StatusOK,
		models.UserPostListResponse{
			Posts: resp,
		})
}

func (h *Handler) getUserFollowerOrFollowing(c echo.Context, fetchFollower bool) error {
	username := c.Param("username")

	limitStr := c.QueryParam("limit")
	limit := int64(DefaultFollowViewLimit)
	if limitStr != "" {
		limit64, err := strconv.ParseInt(limitStr, 10, 64)
		if err != nil {
			return c.String(400, "invalid limit")
		}
		if limit64 < 0 {
			return c.String(400, "invalid limit")
		}
		limit = limit64
	}
	beforeDateStr := c.QueryParam("before_date")
	var beforeDate *time.Time
	if beforeDateStr != "" {
		beforeDateParsed, err := time.Parse(time.RFC3339, beforeDateStr)
		if err != nil {
			return c.String(400, "invalid before_date")
		}
		beforeDate = &beforeDateParsed
	}

	targetUser, err := users.FindIDByUsername(c.Request().Context(), h.MakeDB(), username)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(404, "user not found")
	}

	var viewerID string
	if c.Get(ContextAuthed).(bool) {
		viewerID = c.Get(ContextUserID).(string)
	}

	var followDB []users.FollowerInfo
	if fetchFollower {
		followDB, err = users.FindFollowers(c.Request().Context(), h.MakeDB(), targetUser.ID, viewerID, beforeDate, limit)
	} else {
		followDB, err = users.FindFollowing(c.Request().Context(), h.MakeDB(), targetUser.ID, viewerID, beforeDate, limit)
	}

	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	follows := []models.UserInfoResponse{}
	for _, follow := range followDB {
		res := models.UserInfoResponse{
			ID:       follow.ID,
			Username: follow.Username,
			Hostname: follow.Host,
			Nickname: follow.Nickname,
			URL:      follow.URL,
			Bio:      follow.Bio,
		}
		if viewerID != "" {
			res.IsFollowing = &follow.IsFollowing
		}
		follows = append(follows, res)
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

func (h *Handler) GetUserFollowers(c echo.Context) error {
	return h.getUserFollowerOrFollowing(c, true)
}

func (h *Handler) GetUserFollowing(c echo.Context) error {
	return h.getUserFollowerOrFollowing(c, false)
}

func (h *Handler) modifyFollow(c echo.Context, isFollow bool) error {
	myUserId := c.Get(ContextUserID).(string)
	targetUsername := c.Param("username")

	// TODO: transaction

	// existence check
	targetUser, err := users.FindIDByUsername(c.Request().Context(), h.MakeDB(), targetUsername)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	if isFollow {
		_, err := h.DB.Exec("INSERT INTO UserFollow (follower_id, followee_id) VALUES (UUID_TO_BIN(?), UUID_TO_BIN(?)) ON DUPLICATE KEY UPDATE id=id", myUserId, targetUser.ID)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	} else {
		_, err := h.DB.Exec("DELETE FROM UserFollow WHERE follower_id=UUID_TO_BIN(?) AND followee_id=UUID_TO_BIN(?)", myUserId, targetUser.ID)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "internal server error")
		}
	}

	return c.NoContent(http.StatusOK)
}

func (h *Handler) FollowAUser(c echo.Context) error {
	return h.modifyFollow(c, true)
}

func (h *Handler) UnfollowAUser(c echo.Context) error {
	return h.modifyFollow(c, false)
}

func (h *Handler) PutUser(c echo.Context) error {
	myUserID := c.Get(ContextUserID).(string)

	var update models.UserProfileUpdate
	if err := c.Bind(&update); err != nil {
		return c.String(http.StatusBadRequest, "invalid request body")
	}

	if err := validate.Struct(update); err != nil {
		return c.String(http.StatusBadRequest, "invalid request body")
	}

	err := users.UpdateProfile(c.Request().Context(), h.MakeDB(), myUserID, &update)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	return c.NoContent(http.StatusOK)
}

func (h *Handler) GetUser(c echo.Context) error {
	userspec := c.Param("username")

	var viewerID string
	if c.Get(ContextAuthed).(bool) {
		viewerID = c.Get(ContextUserID).(string)
	}

	user, err := users.GetProfile(c.Request().Context(), h.MakeDB(), userspec, viewerID)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	if user == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	var userURL string
	if user.URL == nil {
		userURL = users.CreateLocalUserURL(user.Username)
	} else {
		userURL = *user.URL
	}

	labels := []models.UserLabel{}
	for _, label := range user.Labels {
		labels = append(labels, models.UserLabel{
			Key:   label.Key,
			Value: label.Value,
		})
	}

	response := models.UserFullInfoResponse{
		UserInfoResponse: models.UserInfoResponse{
			ID:       user.ID,
			Username: user.Username,
			Hostname: user.Host,
			Nickname: user.Nickname,
			URL:      userURL,
			Bio:      user.Bio,
			Counters: models.UserInfoCounterResponse{
				Following: user.Following,
				Followers: user.Followers,
				Posts:     user.PostCount,
			},
		},
		Labels: labels,
	}

	if viewerID != "" {
		response.IsFollowing = &user.IsFollowingByViewer
	}

	return c.JSON(http.StatusOK, response)
}

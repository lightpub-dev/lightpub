package api

import (
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/users"
	"github.com/lightpub-dev/lightpub/utils"
)

const (
	DefaultUserPostLimit   = 10
	DefaultFollowViewLimit = 10
)

func (h *Handler) GetUserPosts(c echo.Context) error {
	authed := c.Get(ContextAuthed).(bool)
	var viewerUserID db.UUID
	if authed {
		viewerUserID = c.Get(ContextUserID).(db.UUID)
	}
	username := c.Param("username")

	userFinderService := initializeUserFinderService(c, h)
	targetUser, err := userFinderService.FindIDByUsername(username)
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

	userPostService := initializeUserPostService(c, h)
	allPosts, err := userPostService.GetUserPosts(targetUser.ID, viewerUserID, posts.UserPostFetchOptions{
		Limit: limit,
	})
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}

	// convert to response
	fetchPostService := initializePostFetchService(c, h)
	resp := []models.UserPostEntry{}
	for _, post := range allPosts {
		hostname := utils.ConvertSqlHost(targetUser.Host)

		var replyToPostOrURL, repostOfPostOrURL interface{}
		if post.ReplyTo != nil {
			replyToPostOrURL = posts.CreatePostURL(post.ReplyToID.UUID)
		}
		if post.RepostOf != nil {
			repostOfPostOrURL, err = fetchPostService.FetchSinglePostWithDepth(post.RepostOfID.UUID, viewerUserID, 0)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "internal server error")
			}
		}

		resp = append(resp, models.UserPostEntry{
			ID:       post.ID,
			IDString: post.ID.String(),
			Author: models.UserPostEntryAuthor{
				ID:       targetUser.ID.String(),
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

	postCountService := initializePostCountService(c, h)
	for i := range resp {
		// add counts
		if err := postCountService.FillCounts(&resp[i]); err != nil {
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
	limit := DefaultFollowViewLimit
	if limitStr != "" {
		limit64, err := strconv.ParseInt(limitStr, 10, 64)
		if err != nil {
			return c.String(400, "invalid limit")
		}
		if limit64 < 0 {
			return c.String(400, "invalid limit")
		}
		limit = int(limit64)
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

	userFinderService := initializeUserFinderService(c, h)
	targetUser, err := userFinderService.FindIDByUsername(username)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(404, "user not found")
	}

	var viewerID db.UUID
	if c.Get(ContextAuthed).(bool) {
		viewerID = c.Get(ContextUserID).(db.UUID)
	}

	userFollowService := initializeUserFollowService(c, h)
	var followDB []users.FollowerInfo
	if fetchFollower {
		followDB, err = userFollowService.FindFollowers(targetUser.ID, viewerID, beforeDate, limit)
	} else {
		followDB, err = userFollowService.FindFollowing(targetUser.ID, viewerID, beforeDate, limit)
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
			URL:      *follow.URL,
			Bio:      follow.Bio,
		}
		if viewerID != (db.UUID{}) {
			res.IsFollowing = &follow.IsFollowing
		}
		follows = append(follows, res)
	}

	return c.JSON(http.StatusOK, map[string]interface{}{
		"results": follows,
	})
}

func (h *Handler) GetUserFollowers(c echo.Context) error {
	return h.getUserFollowerOrFollowing(c, true)
}

func (h *Handler) GetUserFollowing(c echo.Context) error {
	return h.getUserFollowerOrFollowing(c, false)
}

func (h *Handler) modifyFollow(c echo.Context, isFollow bool) error {
	myUserId := c.Get(ContextUserID).(db.UUID)
	targetUsername := c.Param("username")
	targetUsername = strings.ReplaceAll(targetUsername, "%40", "@")
	targetSpec, err := users.ParseUserSpec(targetUsername)
	if err != nil {
		if errors.Is(err, users.ErrInvalidUserSpec) {
			return c.String(http.StatusBadRequest, "invalid userspec")
		}
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	// TODO: transaction

	// existence check
	userFinderService := initializeUserFinderService(c, h)
	userFollowService := initializeUserFollowService(c, h)
	targetUser, err := userFinderService.FetchUser(targetSpec)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
	}
	if targetUser == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	if isFollow {
		err = userFollowService.Follow(users.NewSpecifierFromID(myUserId), users.NewSpecifierFromID(targetUser.ID))
	} else {
		err = userFollowService.Unfollow(users.NewSpecifierFromID(myUserId), users.NewSpecifierFromID(targetUser.ID))
	}

	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "internal server error")
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
	myUserID := c.Get(ContextUserID).(db.UUID)
	targetUserSpec := c.Param("userspec")

	userFinderService := initializeUserFinderService(c, h)
	userProfileService := initializeUserProfileService(c, h)

	targetUser, err := userFinderService.FindIDByUsername(targetUserSpec)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	if targetUser == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	if targetUser.ID != myUserID {
		return c.String(http.StatusForbidden, "you can't update other's profile")
	}

	var update models.UserProfileUpdate
	if err := c.Bind(&update); err != nil {
		return c.String(http.StatusBadRequest, "invalid request body")
	}

	if err := validate.Struct(update); err != nil {
		return c.String(http.StatusBadRequest, "invalid request body")
	}

	err = userProfileService.UpdateProfile(targetUser.ID, &update)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	return c.NoContent(http.StatusOK)
}

func (h *Handler) GetUser(c echo.Context) error {
	userspec := c.Param("username")

	var viewerID db.UUID
	if c.Get(ContextAuthed).(bool) {
		viewerID = c.Get(ContextUserID).(db.UUID)
	}

	if isJsonLDRequested(c) {
		finderService := initializeUserFinderService(c, h)
		pubUserService := initializePubUserService(c, h)

		spec, err := users.ParseUserSpec(userspec)
		if err != nil {
			return c.String(http.StatusBadRequest, "invalid userspec")
		}
		user, err := finderService.FetchUser(spec)
		if err != nil {
			c.Logger().Error(err)
			return c.String(http.StatusInternalServerError, "internal server error")
		}
		if user == nil {
			return c.String(http.StatusNotFound, "user not found")
		}
		obj, err := pubUserService.CreateUserObject(user)
		if err != nil {
			c.Logger().Error(err)
			return c.String(http.StatusInternalServerError, "internal server error")
		}
		jsonMap, err := obj.Serialize()
		if err != nil {
			c.Logger().Error(err)
			return c.String(http.StatusInternalServerError, "internal server error")
		}
		jsonMap["@context"] = []string{
			"https://www.w3.org/ns/activitystreams",
			"https://w3id.org/security/v1",
		}
		return ResponseActivityJson(c, jsonMap)
	}

	userProfileService := initializeUserProfileService(c, h)
	user, err := userProfileService.GetProfile(userspec, viewerID)
	if err != nil {
		c.Logger().Error(err)
		return c.String(http.StatusInternalServerError, "internal server error")
	}

	if user == nil {
		return c.String(http.StatusNotFound, "user not found")
	}

	var userURL string
	if !user.URI.Valid {
		userURL = users.CreateLocalUserURL(user.Username)
	} else {
		userURL = user.URI.String
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
			ID:       user.ID.String(),
			Username: user.Username,
			Hostname: utils.ConvertSqlHost(user.Host),
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

	if viewerID != (db.UUID{}) {
		response.IsFollowing = &user.IsFollowingByViewer
	}

	return c.JSON(http.StatusOK, response)
}

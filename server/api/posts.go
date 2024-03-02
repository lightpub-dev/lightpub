package api

import (
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
)

func (h *Handler) PostPost(c echo.Context) error {
	var body models.PostRequest
	if err := c.Bind(&body); err != nil {
		return c.String(400, "Bad Request")
	}

	// validate
	if err := validate.Struct(body); err != nil {
		return c.String(400, err.Error())
	}

	if body.Content == nil && body.RepostOf == nil {
		return c.String(400, "Content cannot be empty when not reposting")
	}

	var (
		replyToUUID  *db.UUID
		repostOfUUID *db.UUID
	)
	if body.ReplyTo != nil {
		if err := db.ParseTo(replyToUUID, *body.ReplyTo); err != nil {
			return c.String(http.StatusBadRequest, "Invalid json")
		}
	}
	if body.RepostOf != nil {
		if err := db.ParseTo(repostOfUUID, *body.RepostOf); err != nil {
			return c.String(http.StatusBadRequest, "Invalid json")
		}
	}

	post := posts.CreateRequest{
		PosterID:       c.Get(ContextUserID).(db.UUID),
		PosterUsername: c.Get(ContextUsername).(string),
		Content:        body.Content,
		Privacy:        posts.PrivacyType(body.Privacy),

		ReplyToPostID: replyToUUID,
		RepostID:      repostOfUUID,
	}

	postCreateService := initializePostCreateService(c, h)
	result, err := postCreateService.CreatePost(post)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": result.PostID,
	})
}

func (h *Handler) modPostReaction(c echo.Context, reaction string, isAdd bool) error {
	postIdStr := c.Param("post_id")
	userId := c.Get(ContextUserID).(db.UUID)

	postIdUUID, err := uuid.Parse(postIdStr)
	if err != nil {
		return c.String(400, "Bad Request")
	}
	postId := db.UUID(postIdUUID)

	reactionService := initializePostReactionService(c, h)
	if isAdd {
		err = reactionService.AddPostReaction(postId, userId, reaction)
	} else {
		err = reactionService.RemovePostReaction(postId, userId, reaction)
	}

	if err != nil {
		if errors.Is(err, posts.ErrPostNotFound) {
			return c.String(404, "Post not found")
		}
		if errors.Is(err, posts.ErrReactionNotFound) {
			return c.String(400, "Invalid reaction")
		}
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.NoContent(200)
}

func (h *Handler) PutPostReaction(c echo.Context) error {
	reaction := c.Param("reaction")
	if reaction == "" {
		return c.String(400, "Bad Request")
	}

	return h.modPostReaction(c, reaction, true)
}

func (h *Handler) DeletePostReaction(c echo.Context) error {
	reaction := c.Param("reaction")
	if reaction == "" {
		return c.String(400, "Bad Request")
	}

	return h.modPostReaction(c, reaction, false)
}

func (h *Handler) modPostBookmark(c echo.Context, isAdd, isBookmark bool) error {
	postIdStr := c.Param("post_id")
	userId := c.Get(ContextUserID).(db.UUID)

	postIdUUID, err := uuid.Parse(postIdStr)
	if err != nil {
		return c.String(400, "Bad Request")
	}
	postId := db.UUID(postIdUUID)

	likeService := initializePostLikeService(c, h)

	if isAdd {
		if isBookmark {
			err = likeService.Bookmark(postId, userId)
		} else {
			err = likeService.Favorite(postId, userId)
		}
	} else {
		if isBookmark {
			err = likeService.Unbookmark(postId, userId)
		} else {
			err = likeService.Unfavorite(postId, userId)
		}
	}

	if err != nil {
		if errors.Is(err, posts.ErrPostNotFound) {
			return c.String(404, "Post not found")
		}
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.NoContent(200)
}

func (h *Handler) PutPostFavorite(c echo.Context) error {
	return h.modPostBookmark(c, true, false)
}

func (h *Handler) DeletePostFavorite(c echo.Context) error {
	return h.modPostBookmark(c, false, false)
}

func (h *Handler) PutPostBookmark(c echo.Context) error {
	return h.modPostBookmark(c, true, true)
}

func (h *Handler) DeletePostBookmark(c echo.Context) error {
	return h.modPostBookmark(c, false, true)
}

func (h *Handler) GetPost(c echo.Context) error {
	var viewerUserID db.UUID
	if c.Get(ContextAuthed).(bool) {
		viewerUserID = c.Get(ContextUserID).(db.UUID)
	}

	postIDStr := c.Param("post_id")
	postIDUUID, err := uuid.Parse(postIDStr)
	if err != nil {
		return c.String(400, "Bad Request")
	}
	postID := db.UUID(postIDUUID)

	fetchService := initializePostFetchService(c, h)
	post, err := fetchService.FetchSinglePost(postID, viewerUserID)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if post == nil {
		return c.String(404, "Post not found")
	}

	return c.JSON(200, map[string]interface{}{
		"post": post,
	})
}

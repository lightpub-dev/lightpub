package api

import (
	"net/http"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/reactions"
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
		Content:        &body.Content,
		Privacy:        posts.PrivacyType(body.Privacy),

		ReplyToPostID: replyToUUID,
		RepostID:      repostOfUUID,
	}

	result, err := posts.CreatePost(c.Request().Context(), h.MakeDB(), post)
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

	// find reaction id
	reactionObj, err := reactions.FindReactionByID(c.Request().Context(), h.MakeDB(), reaction)
	if err != nil {
		return c.String(http.StatusNotFound, "Reaction not found")
	}

	// check if post is available to user
	visible, err := posts.IsPostVisibleToUser(c.Request().Context(), h.MakeDB(), postId, userId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if !visible {
		// 404 for privacy reasons
		return c.String(404, "Not Found")
	}

	// find original post if repost
	postId, err = posts.FindOriginalPostID(c.Request().Context(), h.MakeDB(), postId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if isAdd {
		// add a reaction
		var postReaction db.PostReaction
		postReaction.PostID = postId
		postReaction.ReactionID = reactionObj.ID
		postReaction.UserID = userId
		err := h.DB.Create(&postReaction).Error
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	} else {
		// delete a reaction
		err := h.DB.Delete(&db.PostReaction{}, "post_id = ? AND user_id = ? AND reaction_id = ?", postId, userId, reactionObj.ID).Error
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
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

	// check if post is available to user
	visible, err := posts.IsPostVisibleToUser(c.Request().Context(), h.MakeDB(), postId, userId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if !visible {
		// 404 for privacy reasons
		return c.String(404, "Not Found")
	}

	// find original post if repost
	postId, err = posts.FindOriginalPostID(c.Request().Context(), h.MakeDB(), postId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if isAdd {
		// add to favorite
		favorite := db.PostFavorite{
			PostID:     postId,
			UserID:     userId,
			IsBookmark: isBookmark,
		}
		err := h.DB.Create(&favorite).Error
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	} else {
		// delete from favorite
		err := h.DB.Delete(&db.PostFavorite{}, "post_id = ? AND user_id = ? AND is_bookmark", postId, userId, isBookmark).Error
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
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

	post, err := posts.FetchSinglePost(c.Request().Context(), h.MakeDB(), postID, viewerUserID)
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

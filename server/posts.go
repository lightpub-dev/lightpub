package main

import (
	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
)

func postPost(c echo.Context) error {
	var body models.PostRequest
	if err := c.Bind(&body); err != nil {
		return c.String(400, "Bad Request")
	}

	// validate
	if err := validate.Struct(body); err != nil {
		return c.String(400, err.Error())
	}

	post := posts.CreateRequest{
		PosterID:       c.Get(ContextUserID).(string),
		PosterUsername: c.Get(ContextUsername).(string),
		Content:        &body.Content,
		Privacy:        posts.PrivacyType(body.Privacy),
		Poll:           body.Poll,
	}

	result, err := posts.CreatePost(makeDBIO(c), post)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": result.PostID,
	})
}

func postReply(c echo.Context) error {
	var body models.PostRequest
	if err := c.Bind(&body); err != nil {
		return c.String(400, "Bad Request")
	}

	// validate
	if err := validate.Struct(body); err != nil {
		return c.String(400, err.Error())
	}

	post := posts.CreateRequest{
		PosterID:       c.Get(ContextUserID).(string),
		PosterUsername: c.Get(ContextUsername).(string),
		Content:        &body.Content,
		Privacy:        posts.PrivacyType(body.Privacy),
		Poll:           body.Poll,

		ReplyToPostID: c.Param("post_id"),
	}

	result, err := posts.CreatePost(makeDBIO(c), post)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": result.PostID,
	})
}

func postRepost(c echo.Context) error {
	var body models.RepostRequest
	if err := c.Bind(&body); err != nil {
		return c.String(400, "Bad Request")
	}

	// validate
	if err := validate.Struct(body); err != nil {
		return c.String(400, err.Error())
	}

	post := posts.CreateRequest{
		PosterID:       c.Get(ContextUserID).(string),
		PosterUsername: c.Get(ContextUsername).(string),
		Content:        nil,
		Privacy:        posts.PrivacyType(body.Privacy),

		RepostID: c.Param("post_id"),
	}

	result, err := posts.CreatePost(makeDBIO(c), post)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": result.PostID,
	})
}

func postQuote(c echo.Context) error {
	var body models.PostRequest
	if err := c.Bind(&body); err != nil {
		return c.String(400, "Bad Request")
	}

	// validate
	if err := validate.Struct(body); err != nil {
		return c.String(400, err.Error())
	}

	post := posts.CreateRequest{
		PosterID:       c.Get(ContextUserID).(string),
		PosterUsername: c.Get(ContextUsername).(string),
		Content:        &body.Content,
		Privacy:        posts.PrivacyType(body.Privacy),
		Poll:           body.Poll,

		RepostID: c.Param("post_id"),
	}

	result, err := posts.CreatePost(makeDBIO(c), post)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": result.PostID,
	})
}

func modPostReaction(c echo.Context, reaction string, isAdd bool) error {
	postId := c.Param("post_id")
	userId := c.Get(ContextUserID).(string)

	// check if post is available to user
	visible, err := posts.IsPostVisibleToUser(makeDBIO(c), postId, userId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if !visible {
		// 404 for privacy reasons
		return c.String(404, "Not Found")
	}

	if isAdd {
		// add a reaction
		_, err = db.Exec("INSERT INTO PostReaction (post_id,reaction,user_id) VALUES (UUID_TO_BIN(?),?,UUID_TO_BIN(?)) ON DUPLICATE KEY UPDATE reaction=reaction", postId, reaction, userId, reaction)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	} else {
		// delete a reaction
		_, err = db.Exec("DELETE FROM PostReaction WHERE post_id=UUID_TO_BIN(?) AND user_id=UUID_TO_BIN(?) AND reaction=?", postId, userId, reaction)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	}

	return c.NoContent(200)
}

func putPostReaction(c echo.Context) error {
	reaction := c.Param("reaction")
	if reaction == "" {
		return c.String(400, "Bad Request")
	}

	return modPostReaction(c, reaction, true)
}

func deletePostReaction(c echo.Context) error {
	reaction := c.Param("reaction")
	if reaction == "" {
		return c.String(400, "Bad Request")
	}

	return modPostReaction(c, reaction, false)
}

func modPostBookmark(c echo.Context, isAdd, isBookmark bool) error {
	postId := c.Param("post_id")
	userId := c.Get(ContextUserID).(string)

	// check if post is available to user
	visible, err := posts.IsPostVisibleToUser(makeDBIO(c), postId, userId)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	if !visible {
		// 404 for privacy reasons
		return c.String(404, "Not Found")
	}

	if isAdd {
		// add a reaction
		_, err = db.Exec("INSERT INTO PostFavorite (post_id,user_id,is_bookmark) VALUES (UUID_TO_BIN(?),UUID_TO_BIN(?),?) ON DUPLICATE KEY UPDATE post_id=post_id", postId, userId, isBookmark)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	} else {
		// delete a reaction
		_, err = db.Exec("DELETE FROM PostFavorite WHERE post_id=UUID_TO_BIN(?) AND user_id=UUID_TO_BIN(?) AND is_bookmark=?", postId, userId, isBookmark)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	}

	return c.NoContent(200)
}

func putPostFavorite(c echo.Context) error {
	return modPostBookmark(c, true, false)
}

func deletePostFavorite(c echo.Context) error {
	return modPostBookmark(c, false, false)
}

func putPostBookmark(c echo.Context) error {
	return modPostBookmark(c, true, true)
}

func deletePostBookmark(c echo.Context) error {
	return modPostBookmark(c, false, true)
}

func getPost(c echo.Context) error {
	viewerUserID := ""
	if c.Get(ContextAuthed).(bool) {
		viewerUserID = c.Get(ContextUserID).(string)
	}

	postID := c.Param("post_id")
	post, err := posts.FetchSinglePost(makeDBIO(c), postID, viewerUserID)
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

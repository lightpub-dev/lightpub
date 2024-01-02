package main

import (
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/utils"
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

	postId, err := utils.GenerateUUIDString()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	currentTime := time.Now()
	dbPost := models.Post{
		ID:          postId,
		PosterID:    c.Get(ContextUserID).(string),
		Content:     body.Content,
		InsertedAt:  currentTime,
		CreatedAt:   currentTime,
		Privacy:     body.Privacy,
		ScheduledAt: body.ScheduledAt,
	}
	posterUsername := c.Get(ContextUsername).(string)

	// insert into db
	tx, err := db.Beginx()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}
	defer tx.Rollback()

	// insert Post
	_, err = tx.NamedExec("INSERT INTO Post (id,poster_id,content,inserted_at,created_at,privacy,scheduled_at) VALUES (UUID_TO_BIN(:id),UUID_TO_BIN(:poster_id),:content,:inserted_at,:created_at,:privacy,:scheduled_at)", dbPost)
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// insert Hashtags (if any)
	hashtags := posts.FindHashtags(body.Content)
	for _, hashtag := range hashtags {
		_, err = tx.Exec("INSERT INTO PostHashtag (post_id,hashtag_name) VALUES (UUID_TO_BIN(?),?)", postId, hashtag)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	}

	// insert Poll (if any)
	if body.Poll != nil {
		pollId, err := utils.GenerateUUIDString()
		if err != nil {
			return c.String(500, "Internal Server Error")
		}
		dbPoll := models.PostPoll{
			ID:            pollId,
			AllowMultiple: body.Poll.AllowMultiple,
			Due:           body.Poll.Due,
		}
		_, err = tx.NamedExec("INSERT INTO PostPoll (id,allow_multiple,due) VALUES (UUID_TO_BIN(:id),:allow_multiple,:due)", dbPoll)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}

		// insert PollChoices
		for _, choice := range body.Poll.Choices {
			_, err = tx.Exec("INSERT INTO PollChoice (poll_id,title,count) VALUES (UUID_TO_BIN(?),?,0)", pollId, choice)
			if err != nil {
				c.Logger().Error(err)
				return c.String(500, "Internal Server Error")
			}
		}
	}

	// insert mentions (if any)
	mentions := posts.FindMentions(body.Content)
	for _, mention := range mentions {
		_, err = tx.Exec("INSERT INTO PostMention (post_id,target_user_id) VALUES (UUID_TO_BIN(?),UUID_TO_BIN(?))", postId, mention)
		if err != nil {
			c.Logger().Error(err)
			return c.String(500, "Internal Server Error")
		}
	}

	// commit
	err = tx.Commit()
	if err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	// publish to timeline
	// TODO: this should be done asynchronously
	if err := posts.RegisterToTimeline(c.Request().Context(), db, rdb, dbPost, posterUsername,
		config.MyHostname, // host should be always local because only local user can post
		hashtags, mentions); err != nil {
		c.Logger().Error(err)
		return c.String(500, "Internal Server Error")
	}

	return c.JSON(201, map[string]interface{}{
		"id": postId,
	})
}

func modPostReaction(c echo.Context, reaction string, isAdd bool) error {
	postId := c.Param("post_id")
	userId := c.Get(ContextUserID).(string)

	// check if post is available to user
	visible, err := posts.IsPostVisibleToUser(db, postId, userId)
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
	visible, err := posts.IsPostVisibleToUser(db, postId, userId)
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

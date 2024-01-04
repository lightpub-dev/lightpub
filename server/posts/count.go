package posts

import "github.com/lightpub-dev/lightpub/db"

func CountReply(dbio *db.DBIO, postID string) (int64, error) {
	var count int64
	err := dbio.GetContext(dbio.Ctx, &count, `
		SELECT COUNT(*) FROM Post
		WHERE reply_to=UUID_TO_BIN(?)
		  AND scheduled_at IS NULL
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountFavorite(dbio *db.DBIO, postID string) (int64, error) {
	var count int64
	err := dbio.GetContext(dbio.Ctx, &count, `
		SELECT COUNT(*) FROM PostFavorite
		WHERE post_id=UUID_TO_BIN(?)
		  AND is_bookmark=0
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountRepost(dbio *db.DBIO, postID string) (int64, error) {
	var count int64
	err := dbio.GetContext(dbio.Ctx, &count, `
		SELECT COUNT(*) FROM Post
		WHERE repost_of=UUID_TO_BIN(?)
		  AND content IS NULL
		  AND scheduled_at IS NULL
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountQuote(dbio *db.DBIO, postID string) (int64, error) {
	var count int64
	err := dbio.GetContext(dbio.Ctx, &count, `
		SELECT COUNT(*) FROM Post
		WHERE repost_of=UUID_TO_BIN(?)
		  AND content IS NOT NULL
		  AND scheduled_at IS NULL
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

type CountFillable interface {
	PostID() string
	UpdateCounts(reply, favorite, repost, quote int64)
}

func FillCounts(dbio *db.DBIO, fillable CountFillable) error {
	postID := fillable.PostID()
	reply, err := CountReply(dbio, postID)
	if err != nil {
		return err
	}
	favorite, err := CountFavorite(dbio, postID)
	if err != nil {
		return err
	}
	repost, err := CountRepost(dbio, postID)
	if err != nil {
		return err
	}
	quote, err := CountQuote(dbio, postID)
	if err != nil {
		return err
	}
	fillable.UpdateCounts(reply, favorite, repost, quote)
	return nil
}

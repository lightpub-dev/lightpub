package posts

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

func CountReply(ctx context.Context, conn db.DBConn, postID string) (int64, error) {
	var count int64
	err := conn.DB().GetContext(ctx, &count, `
		SELECT COUNT(*) FROM Post
		WHERE reply_to=UUID_TO_BIN(?)
		  AND scheduled_at IS NULL
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountFavorite(ctx context.Context, conn db.DBConn, postID string) (int64, error) {
	var count int64
	err := conn.DB().GetContext(ctx, &count, `
		SELECT COUNT(*) FROM PostFavorite
		WHERE post_id=UUID_TO_BIN(?)
		  AND is_bookmark=0
	`, postID)
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountRepost(ctx context.Context, conn db.DBConn, postID string) (int64, error) {
	var count int64
	err := conn.DB().GetContext(ctx, &count, `
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

func CountQuote(ctx context.Context, conn db.DBConn, postID string) (int64, error) {
	var count int64
	err := conn.DB().GetContext(ctx, &count, `
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

type reactionCountRow struct {
	Reaction string `db:"reaction"`
	Count    int64  `db:"count"`
}

func CountReactions(ctx context.Context, conn db.DBConn, postID string) (models.ReactionCountMap, error) {
	var rows []reactionCountRow
	err := conn.DB().SelectContext(ctx, &rows, `
	SELECT pr.reaction, count(pr.user_id) AS count
	FROM PostReaction pr
	WHERE
		pr.post_id=UUID_TO_BIN(?)
	GROUP BY pr.reaction;
	`, postID)
	if err != nil {
		return nil, err
	}

	reactions := make(models.ReactionCountMap)
	for _, row := range rows {
		reactions[row.Reaction] = row.Count
	}
	return reactions, nil
}

type CountFillable interface {
	PostID() string
	UpdateCounts(reply, favorite, repost, quote int64, reactions models.ReactionCountMap)
}

func FillCounts(ctx context.Context, conn db.DBConn, fillable CountFillable) error {
	postID := fillable.PostID()
	reply, err := CountReply(ctx, conn, postID)
	if err != nil {
		return err
	}
	favorite, err := CountFavorite(ctx, conn, postID)
	if err != nil {
		return err
	}
	repost, err := CountRepost(ctx, conn, postID)
	if err != nil {
		return err
	}
	quote, err := CountQuote(ctx, conn, postID)
	if err != nil {
		return err
	}
	reactions, err := CountReactions(ctx, conn, postID)
	if err != nil {
		return err
	}
	fillable.UpdateCounts(reply, favorite, repost, quote, reactions)
	return nil
}

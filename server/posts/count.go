package posts

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

func CountReply(ctx context.Context, conn db.DBConn, postID db.UUID) (int64, error) {
	var count int64
	err := conn.DB().Model(&db.Post{}).Where("reply_to_id = ?", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountFavorite(ctx context.Context, conn db.DBConn, postID db.UUID) (int64, error) {
	var count int64
	err := conn.DB().Model(&db.PostFavorite{}).Where("post_id = ? AND is_bookmark = 0", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountRepost(ctx context.Context, conn db.DBConn, postID db.UUID) (int64, error) {
	var count int64
	err := conn.DB().Model(&db.Post{}).Where("repost_of_id = ? AND content IS NULL", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func CountQuote(ctx context.Context, conn db.DBConn, postID db.UUID) (int64, error) {
	var count int64
	err := conn.DB().Model(&db.Post{}).Where("repost_of_id = ? AND content IS NOT NULL", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

type reactionCountRow struct {
	Reaction string
	Count    int64
}

func CountReactions(ctx context.Context, conn db.DBConn, postID db.UUID) (models.ReactionCountMap, error) {
	var rows []reactionCountRow
	err := conn.DB().Model(&db.PostReaction{}).Select("reaction", "COUNT(user_id) AS count").Where("post_id = ?", postID).Group("reaction").Find(&rows).Error
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
	PostID() db.UUID
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

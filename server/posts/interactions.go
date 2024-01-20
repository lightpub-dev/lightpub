package posts

import "github.com/lightpub-dev/lightpub/db"

type InteractionFillable interface {
	FillInteractions(repostedByMe, favoritedByMe, bookmarkedByMe bool)
	PostID() db.UUID
}

func FillInteraction(conn db.DBConn, interactedByUserID db.UUID, fillable InteractionFillable) error {
	postID := fillable.PostID()

	var repostedByMe, favoritedByMe, bookmarkedByMe int64
	err := conn.DB().Model(&db.Post{}).Where("repost_of_id = ? AND poster_id = ? AND content IS NULL", postID, interactedByUserID).Count(&repostedByMe).Error
	if err != nil {
		return err
	}

	err = conn.DB().Model(&db.PostFavorite{}).Where("post_id = ? AND user_id = ? AND is_bookmark = 0", postID, interactedByUserID).Count(&favoritedByMe).Error
	if err != nil {
		return err
	}

	err = conn.DB().Model(&db.PostFavorite{}).Where("post_id = ? AND user_id = ? AND is_bookmark = 1", postID, interactedByUserID).Count(&bookmarkedByMe).Error
	if err != nil {
		return err
	}

	fillable.FillInteractions(repostedByMe > 0, favoritedByMe > 0, bookmarkedByMe > 0)
	return nil
}

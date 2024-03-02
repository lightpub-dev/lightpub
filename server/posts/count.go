package posts

import (
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
)

type PostCountService interface {
	CountReply(postID db.UUID) (int64, error)
	CountFavorite(postID db.UUID) (int64, error)
	CountRepost(postID db.UUID) (int64, error)
	CountQuote(postID db.UUID) (int64, error)
	CountReactions(postID db.UUID) (models.ReactionCountMap, error)
	FillCounts(fillable CountFillable) error
}

type DBPostCountService struct {
	conn db.DBConn
}

func ProvideDBPostCountService(conn db.DBConn) *DBPostCountService {
	return &DBPostCountService{conn}
}

func (s *DBPostCountService) CountReply(postID db.UUID) (int64, error) {
	var count int64
	err := s.conn.DB.Model(&db.Post{}).Where("reply_to_id = ?", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func (s *DBPostCountService) CountFavorite(postID db.UUID) (int64, error) {
	var count int64
	err := s.conn.DB.Model(&db.PostFavorite{}).Where("post_id = ? AND is_bookmark = 0", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func (s *DBPostCountService) CountRepost(postID db.UUID) (int64, error) {
	var count int64
	err := s.conn.DB.Model(&db.Post{}).Where("repost_of_id = ? AND content IS NULL", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

func (s *DBPostCountService) CountQuote(postID db.UUID) (int64, error) {
	var count int64
	err := s.conn.DB.Model(&db.Post{}).Where("repost_of_id = ? AND content IS NOT NULL", postID).Count(&count).Error
	if err != nil {
		return 0, err
	}
	return count, nil
}

type reactionCountRow struct {
	Reaction string
	Count    int64
}

func (s *DBPostCountService) CountReactions(postID db.UUID) (models.ReactionCountMap, error) {
	var rows []reactionCountRow
	err := s.conn.DB.Model(&db.PostReaction{}).Select("post_reactions.reaction_id", "COUNT(post_reactions.user_id) AS count", "Reaction.name AS reaction").Joins("Reaction").Where("post_reactions.post_id = ?", postID).Group("reaction_id").Find(&rows).Error
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

func (s *DBPostCountService) FillCounts(fillable CountFillable) error {
	postID := fillable.PostID()
	reply, err := s.CountReply(postID)
	if err != nil {
		return err
	}
	favorite, err := s.CountFavorite(postID)
	if err != nil {
		return err
	}
	repost, err := s.CountRepost(postID)
	if err != nil {
		return err
	}
	quote, err := s.CountQuote(postID)
	if err != nil {
		return err
	}
	reactions, err := s.CountReactions(postID)
	if err != nil {
		return err
	}
	fillable.UpdateCounts(reply, favorite, repost, quote, reactions)
	return nil
}

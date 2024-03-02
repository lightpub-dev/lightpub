package posts

import (
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/reactions"
	"gorm.io/gorm"
)

type PostReactionService interface {
	AddPostReaction(postID db.UUID, userID db.UUID, reactionName string) error
	RemovePostReaction(postID db.UUID, userID db.UUID, reactionName string) error
}

type DBPostReactionService struct {
	conn       db.DBConn
	reaction   reactions.FindReactionService
	visibility PostVisibilityService
	fetch      PostFetchService
}

func ProvideDBPostReactionService(
	conn db.DBConn, reaction reactions.FindReactionService,
	visibility PostVisibilityService, fetch PostFetchService,
) *DBPostReactionService {
	return &DBPostReactionService{conn, reaction, visibility, fetch}
}

var (
	ErrPostNotFound     = errors.New("post not found")
	ErrUserNotFound     = errors.New("user not found")
	ErrReactionNotFound = errors.New("reaction not found")
)

func (s *DBPostReactionService) AddPostReaction(
	postID db.UUID, userID db.UUID, reactionName string,
) error {
	// find reaction id
	reactionObj, err := s.reaction.FindReactionByID(reactionName)
	if err != nil {
		return ErrReactionNotFound
	}

	// check if post is visible to user
	visible, err := s.visibility.IsPostVisibleToUser(postID, userID)
	if err != nil {
		return err
	}
	if !visible {
		return ErrPostNotFound
	}

	// find original post if repost
	originalPostID, err := s.fetch.FindOriginalPostID(postID)
	if err != nil {
		return err
	}

	if err := s.conn.DB.Create(&db.PostReaction{
		PostID:     originalPostID,
		UserID:     userID,
		ReactionID: reactionObj.ID,
	}).Error; err != nil {
		return err
	}

	return nil
}

func (s *DBPostReactionService) RemovePostReaction(
	postID db.UUID, userID db.UUID, reactionName string,
) error {
	// find reaction id
	reactionObj, err := s.reaction.FindReactionByID(reactionName)
	if err != nil {
		return ErrReactionNotFound
	}

	// check if post is visible to user
	visible, err := s.visibility.IsPostVisibleToUser(postID, userID)
	if err != nil {
		return err
	}
	if !visible {
		return ErrPostNotFound
	}

	// find original post if repost
	originalPostID, err := s.fetch.FindOriginalPostID(postID)
	if err != nil {
		return err
	}

	// find PostReaction object
	var postReaction *db.PostReaction
	if err := s.conn.DB.Where("post_id = ? AND user_id = ? AND reaction_id = ?", originalPostID, userID, reactionObj.ID).First(postReaction).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// no reaction found. nothing to do
			return nil
		}
		return err
	}

	if err := s.conn.DB.Delete(postReaction).Error; err != nil {
		return err
	}

	return nil
}

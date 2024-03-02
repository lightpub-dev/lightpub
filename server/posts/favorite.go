package posts

import (
	"github.com/lightpub-dev/lightpub/db"
)

type PostLikeService interface {
	Favorite(postID, userID db.UUID) error
	Unfavorite(postID, userID db.UUID) error
	Bookmark(postID, userID db.UUID) error
	Unbookmark(postID, userID db.UUID) error
}

type DBPostLikeService struct {
	conn       db.DBConn
	visibility PostVisibilityService
	fetch      PostFetchService
}

func ProvideDBPostLikeService(
	conn db.DBConn, visibility PostVisibilityService, fetch PostFetchService,
) *DBPostLikeService {
	return &DBPostLikeService{conn, visibility, fetch}
}

func (s *DBPostLikeService) findTargetPost(postID, userID db.UUID) (db.UUID, error) {
	// check if post is available to user
	visible, err := s.visibility.IsPostVisibleToUser(postID, userID)
	if err != nil {
		return db.UUID{}, err
	}

	if !visible {
		return db.UUID{}, ErrPostNotFound
	}

	// find original post if repost
	postID, err = s.fetch.FindOriginalPostID(postID)
	if err != nil {
		return db.UUID{}, err
	}

	return postID, nil
}

func (s *DBPostLikeService) Favorite(postID, userID db.UUID) error {
	postID, err := s.findTargetPost(postID, userID)
	if err != nil {
		return err
	}

	return s.conn.DB.Create(&db.PostFavorite{
		PostID:     postID,
		UserID:     userID,
		IsBookmark: false,
	}).Error
}

func (s *DBPostLikeService) Unfavorite(postID, userID db.UUID) error {
	postID, err := s.findTargetPost(postID, userID)
	if err != nil {
		return err
	}

	return s.conn.DB.Delete(&db.PostFavorite{
		PostID:     postID,
		UserID:     userID,
		IsBookmark: false,
	}).Error
}

func (s *DBPostLikeService) Bookmark(postID, userID db.UUID) error {
	postID, err := s.findTargetPost(postID, userID)
	if err != nil {
		return err
	}

	return s.conn.DB.Create(&db.PostFavorite{
		PostID:     postID,
		UserID:     userID,
		IsBookmark: true,
	}).Error
}

func (s *DBPostLikeService) Unbookmark(postID, userID db.UUID) error {
	postID, err := s.findTargetPost(postID, userID)
	if err != nil {
		return err
	}

	return s.conn.DB.Delete(&db.PostFavorite{
		PostID:     postID,
		UserID:     userID,
		IsBookmark: true,
	}).Error
}

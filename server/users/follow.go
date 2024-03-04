package users

import (
	"database/sql"
	"errors"
	"fmt"
	"log"
	"time"

	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/config"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var (
	ErrFollowerNotFound = errors.New("follower not found")
	ErrFolloweeNotFound = errors.New("followee not found")
)

type UserFollowService interface {
	IsFollowedBy(followerID db.UUID, followeeID db.UUID) (bool, error)
	FindFollowers(followeeID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error)
	FindFollowing(followerID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error)
	FindFollowersInboxes(followeeID db.UUID) ([]FollowerInbox, error)
	Follow(followerSpec Specifier, followeeSpec Specifier) error
	Unfollow(followerSpec Specifier, followeeSpec Specifier) error

	ProcessAccept(accept vocab.ActivityStreamsAccept) error
	ProcessReject(reject vocab.ActivityStreamsReject) error
	ProcessUndo(undo vocab.ActivityStreamsUndo) error
	ProcessFollow(follow vocab.ActivityStreamsFollow) error
}

type DBUserFollowService struct {
	conn       db.DBConn
	pubFollow  *PubFollowService
	userFinder UserFinderService
	idGetter   pub.IDGetterService
}

func ProvideDBUserFollowService(conn db.DBConn, pubFollow *PubFollowService, userFinder UserFinderService, idGetter pub.IDGetterService) *DBUserFollowService {
	return &DBUserFollowService{conn: conn, pubFollow: pubFollow, userFinder: userFinder, idGetter: idGetter}
}

type FollowerInbox struct {
	UserID      db.UUID `gorm:"column:id"`
	Inbox       sql.NullString
	SharedInbox sql.NullString
}

func (s *DBUserFollowService) IsFollowedBy(followerID db.UUID, followeeID db.UUID) (bool, error) {
	conn := s.conn.DB

	var count int64
	err := conn.Model(&db.UserFollow{}).Where("follower_id = ? AND followee_id = ?", followerID, followeeID).Count(&count).Error
	if err != nil {
		return false, err
	}

	return count > 0, nil
}

type FollowerInfo struct {
	ID          string  `json:"id"`
	Username    string  `json:"username"`
	Host        string  `json:"host"`
	URL         *string `json:"url"` // always non-nil after fillInLocalURL
	Nickname    string  `json:"nickname"`
	Bio         string  `json:"bio"`
	IsFollowing bool    `json:"is_following"`
}

func CreateLocalUserURL(username string) string {
	return fmt.Sprintf("%s/user/%s", config.BaseURL, username)
}

func fillInLocalURL(follower *FollowerInfo) {
	if follower.URL == nil {
		localUrl := CreateLocalUserURL(follower.Username)
		follower.URL = &localUrl
	}
}

func (s *DBUserFollowService) FindFollowers(followeeID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	conn := s.conn.DB

	var (
		followers []FollowerInfo
		tx        *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("followee_id = ?", followeeID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.follower_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("followee_id = ?", followeeID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC")
	if limit >= 0 {
		tx = tx.Limit(limit)
	}

	err := tx.Find(&followers).Error
	if err != nil {
		return nil, err
	}

	for i, follower := range followers {
		fillInLocalURL(&follower)
		followers[i] = follower
	}

	return followers, nil
}

func (s *DBUserFollowService) FindFollowing(followerID db.UUID, viewerID db.UUID, beforeDate *time.Time, limit int) ([]FollowerInfo, error) {
	conn := s.conn.DB

	var (
		followings []FollowerInfo
		tx         *gorm.DB
	)
	if viewerID == (db.UUID{}) {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Where("follower_id = ?", followerID).Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio")
	} else {
		tx = conn.Model(&db.UserFollow{}).Joins("JOIN users ON users.id = user_follows.followee_id").Joins("JOIN user_profiles ON user_profiles.user_id = users.id").Select("users.id AS id, users.username, users.host, users.url, users.nickname, user_profiles.bio, COUNT(user_follows.follower_id) AS is_following").Where("follower_id = ?", followerID)
	}

	if beforeDate != nil {
		tx = tx.Where("users.created_at < ?", beforeDate)
	}

	tx = tx.Order("users.created_at DESC")
	if limit >= 0 {
		tx = tx.Limit(limit)
	}

	err := tx.Find(&followings).Error
	if err != nil {
		return nil, err
	}

	for i, follower := range followings {
		fillInLocalURL(&follower)
		followings[i] = follower
	}

	return followings, nil
}

func (s *DBUserFollowService) FindFollowersInboxes(followeeID db.UUID) ([]FollowerInbox, error) {
	conn := s.conn.DB

	var inboxes []FollowerInbox
	// TODO: follower が多すぎると IN の制限でエラーにならない?
	err := conn.Model(&db.User{}).Select("id, inbox, shared_inbox").Where("id IN (SELECT follower_id FROM user_follows WHERE followee_id = ?)", followeeID).Find(&inboxes).Error
	if err != nil {
		return nil, err
	}

	return inboxes, nil
}

func (s *DBUserFollowService) Follow(followerSpec Specifier, followeeSpec Specifier) error {
	conn := s.conn.DB

	follower, err := s.userFinder.FetchUser(followerSpec)
	if err != nil {
		return err
	}
	if follower == nil {
		return ErrFollowerNotFound
	}

	followee, err := s.userFinder.FetchUser(followeeSpec)
	if err != nil {
		return err
	}
	if followee == nil {
		return ErrFolloweeNotFound
	}

	// follower must be a local user
	if follower.Host.Valid {
		return errors.New("follower must be a local user")
	}

	if !followee.Host.Valid {
		// local user
		follow := db.UserFollow{
			FollowerID: follower.ID,
			FolloweeID: followee.ID,
		}

		if err := conn.Create(&follow).Error; err != nil {
			return err
		}

		return nil
	}

	// remote user
	// save follow request
	// check existing request
	tx := conn.Begin()
	defer tx.Rollback()
	var req db.UserFollowRequest
	if err = tx.Where("follower_id = ? AND followee_id = ?", follower.ID, followee.ID).First(&req).Error; err != nil {
		if !errors.Is(err, gorm.ErrRecordNotFound) {
			return err
		}

		req = db.UserFollowRequest{
			ID:         db.MustGenerateUUID(),
			FollowerID: follower.ID,
			FolloweeID: followee.ID,
			Incoming:   false,
			URI:        sql.NullString{},
		}
		if err := s.conn.DB.Create(&req).Error; err != nil {
			return err
		}
	}

	if err := tx.Commit().Error; err != nil {
		return err
	}

	// send Follow to remote inbox
	reqID, err := s.idGetter.GetFollowRequestID(&req)
	if err != nil {
		return err
	}
	if err := s.pubFollow.SendFollowRequest(reqID, follower, followee); err != nil {
		return err
	}

	return nil
}

func (s *DBUserFollowService) Unfollow(followerSpec Specifier, followeeSpec Specifier) error {
	conn := s.conn.DB

	follower, err := s.userFinder.FetchUser(followerSpec)
	if err != nil {
		return err
	}
	if follower == nil {
		return ErrFollowerNotFound
	}

	followee, err := s.userFinder.FetchUser(followeeSpec)
	if err != nil {
		return err
	}
	if followee == nil {
		return ErrFolloweeNotFound
	}

	tx := conn.Begin()
	defer tx.Rollback()
	err = tx.Delete(&db.UserFollow{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error
	if err != nil {
		return err
	}
	err = tx.Delete(&db.UserFollowRequest{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error
	if err != nil {
		return err
	}

	if err := tx.Commit().Error; err != nil {
		return err
	}

	if followee.Host.Valid {
		// followee is a remote user
		// send a reject message
		if err := s.pubFollow.SendUnfollowRequest(rejectUnfollowRequest{
			Follower: follower,
			Followee: followee,
		}); err != nil {
			log.Printf("failed to send unfollow request: %v", err)
		}
	}

	return nil
}

func (s *DBUserFollowService) ProcessAccept(accept vocab.ActivityStreamsAccept) error {
	parsedAccept, err := s.pubFollow.ProcessAccept(accept)
	if err != nil {
		return fmt.Errorf("failed to analyze accept request: %w", err)
	}

	tx := s.conn.DB.Begin()
	defer tx.Rollback()
	var (
		req db.UserFollowRequest
	)
	if parsedAccept.ReqID != nil {
		// find by request id
		var localReqID string
		localReqID, err = s.idGetter.ExtractLocalFollowRequestID(parsedAccept.ReqID.String())
		if err != nil {
			return err
		}
		if localReqID != "" {
			// it is a local request
			// find by id
			err = tx.Model(&db.UserFollowRequest{}).Where("id = ?", localReqID).First(&req).Error
		} else {
			// it is a remote request
			// find by uri
			err = tx.Model(&db.UserFollowRequest{}).Where("uri = ?", parsedAccept.ReqID.String()).First(&req).Error
		}
	} else {
		// find by follower and followee id
		follower, err2 := s.userFinder.FetchUser(NewSpecifierFromURI(parsedAccept.ActorURI))
		if err2 != nil {
			return err2
		}
		if follower == nil {
			return ErrFollowerNotFound
		}
		followee, err2 := s.userFinder.FetchUser(NewSpecifierFromURI(parsedAccept.ObjectURI))
		if err2 != nil {
			return err2
		}
		if followee == nil {
			return ErrFolloweeNotFound
		}

		err = tx.Model(&db.UserFollowRequest{}).Where("follower_id = ? AND followee_id = ?", follower.ID, followee.ID).First(&req).Error
	}

	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return errors.New("follow request not found")
		}
		return err
	}

	// delete request and insert actual follow
	if err := tx.Delete(&req).Error; err != nil {
		return err
	}

	follow := &db.UserFollow{
		FollowerID: req.FollowerID,
		FolloweeID: req.FolloweeID,
	}
	if err := tx.Clauses(
		clause.OnConflict{
			DoNothing: true,
		},
	).Create(follow).Error; err != nil {
		return err
	}

	return tx.Commit().Error
}

func (s *DBUserFollowService) ProcessReject(reject vocab.ActivityStreamsReject) error {
	parsedReject, err := s.pubFollow.ProcessReject(reject)
	if err != nil {
		return fmt.Errorf("failed to analyze reject request: %w", err)
	}

	follower, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedReject.ActorURI))
	if err != nil {
		return err
	}
	if follower == nil {
		return ErrFollowerNotFound
	}
	followee, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedReject.ObjectURI))
	if err != nil {
		return err
	}
	if followee == nil {
		return ErrFolloweeNotFound
	}

	tx := s.conn.DB.Begin()
	defer tx.Rollback()

	// delete request and follow
	if err := tx.Delete(&db.UserFollowRequest{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error; err != nil {
		return err
	}
	if err := tx.Delete(&db.UserFollow{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error; err != nil {
		return err
	}

	return tx.Commit().Error
}

func (s *DBUserFollowService) ProcessUndo(undo vocab.ActivityStreamsUndo) error {
	parsedUndo, err := s.pubFollow.ProcessUndo(undo)
	if err != nil {
		return fmt.Errorf("failed to analyze undo request: %w", err)
	}

	// find by request id
	var localReqID string
	localReqID, err = s.idGetter.ExtractLocalFollowRequestID(parsedUndo.RequestID.String())
	if err != nil {
		return err
	}

	tx := s.conn.DB.Begin()
	defer tx.Rollback()

	if localReqID != "" {
		// it is a local request
		// find by id
		err = tx.Delete(&db.UserFollowRequest{}, "id = ?", localReqID).Error
	} else {
		// it is a remote request
		// find by uri
		err = tx.Delete(&db.UserFollowRequest{}, "uri = ?", parsedUndo.RequestID.String()).Error
	}
	if err != nil {
		return fmt.Errorf("failed to delete follow request: %w", err)
	}

	if parsedUndo.ActorURI == nil && parsedUndo.ObjectURI == nil {
		// if actor and object are not set, finish here
		return tx.Commit().Error
	}

	// otherwise, delete actual follow too
	follower, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedUndo.ActorURI))
	if err != nil {
		return err
	}
	if follower == nil {
		return ErrFollowerNotFound
	}
	followee, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedUndo.ObjectURI))
	if err != nil {
		return err
	}

	// delete follow
	err = tx.Delete(&db.UserFollow{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error
	if err != nil {
		return err
	}

	return tx.Commit().Error
}

func (s *DBUserFollowService) ProcessFollow(follow vocab.ActivityStreamsFollow) error {
	parsedFollow, err := s.pubFollow.ProcessFollow(follow)
	if err != nil {
		return fmt.Errorf("failed to analyze follow request: %w", err)
	}

	follower, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedFollow.ActorURI))
	if err != nil {
		return err
	}
	if follower == nil {
		return ErrFollowerNotFound
	}
	followee, err := s.userFinder.FetchUser(NewSpecifierFromURI(parsedFollow.ObjectURI))
	if err != nil {
		return err
	}
	if followee == nil {
		return ErrFolloweeNotFound
	}

	// check if followee is a local user
	if followee.Host.Valid {
		return errors.New("ProcessFollow: followee must be a local user")
	}

	tx := s.conn.DB.Begin()
	defer tx.Rollback()

	// delete existing requests
	if err := tx.Delete(&db.UserFollowRequest{}, "follower_id = ? AND followee_id = ?", follower.ID, followee.ID).Error; err != nil {
		return err
	}

	req := db.UserFollowRequest{
		ID:         db.MustGenerateUUID(),
		URI:        sql.NullString{String: parsedFollow.RequestID.String(), Valid: true},
		FollowerID: follower.ID,
		FolloweeID: followee.ID,
		Incoming:   true,
	}
	if err := tx.Create(&req).Error; err != nil {
		return err
	}
	reqID := req.ID

	if err := tx.Commit().Error; err != nil {
		return err
	}

	// TODO: check if user is auto-accepting follows

	// send accept
	go func() {
		var req db.UserFollowRequest
		if err := s.conn.DB.Joins("Follower").Joins("Followee").First(&req, "user_follow_requests.id = ?", reqID).Error; err != nil {
			log.Printf("failed to find follow request: %v", err)
			return
		}
		if err := s.pubFollow.SendAcceptFollowRequest(&req); err != nil {
			log.Printf("failed to send accept follow request: %v", err)
		}
		if err := s.conn.DB.Delete(&db.UserFollowRequest{}, "id = ?", reqID).Error; err != nil {
			log.Printf("failed to delete follow request: %v", err)
		}
	}()

	return nil
}

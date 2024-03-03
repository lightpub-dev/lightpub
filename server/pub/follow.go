package pub

import (
	"errors"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
)

type PubFollowService struct {
	getter IDGetterService
}

var (
	ErrUserInboxNotValid = errors.New("user inbox not set or valid")
)

func (s *PubFollowService) SendFollowRequest(follower, following *db.User) error {
	if !following.Inbox.Valid {
		return ErrUserInboxNotValid
	}
	// inbox := following.Inbox.String
	// follow, err := s.createFollowRequest(follower, following)
	// if err != nil {
	// 	return err
	// }
	return nil
}

func (s *PubFollowService) createFollowRequest(follower, following *db.User) (vocab.ActivityStreamsFollow, error) {
	follow := streams.NewActivityStreamsFollow()

	followerURI, err := s.getter.GetUserID(follower, "")
	if err != nil {
		return nil, err
	}
	followingURI, err := s.getter.GetUserID(following, "")
	if err != nil {
		return nil, err
	}

	actorProp := streams.NewActivityStreamsActorProperty()
	actorProp.AppendIRI(followerURI)
	follow.SetActivityStreamsActor(actorProp)

	objectProp := streams.NewActivityStreamsObjectProperty()
	objectProp.AppendIRI(followingURI)
	follow.SetActivityStreamsObject(objectProp)

	return follow, nil
}

func (s *PubFollowService) SendAcceptFollowRequest(req *db.UserFollowRequest) error {
	if req.Followee.Inbox.Valid {
		return ErrUserInboxNotValid
	}
	return nil
}

func (s *PubFollowService) createAccept(req *db.UserFollowRequest) (vocab.ActivityStreamsAccept, error) {
	accept := streams.NewActivityStreamsAccept()

	reqID, err := s.getter.GetFollowRequestID(req)
	if err != nil {
		return nil, err
	}
	objectProp := streams.NewActivityStreamsObjectProperty()
	objectProp.AppendIRI(reqID)
	accept.SetActivityStreamsObject(objectProp)

	actorURI, err := s.getter.GetUserID(&req.Follower, "")
	if err != nil {
		return nil, err
	}
	actorProp := streams.NewActivityStreamsActorProperty()
	actorProp.AppendIRI(actorURI)
	accept.SetActivityStreamsActor(actorProp)

	return accept, nil
}

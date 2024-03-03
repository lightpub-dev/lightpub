package users

import (
	"errors"
	"net/url"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
)

type PubFollowService struct {
	getter pub.IDGetterService
	req    pub.RequesterService
}

func ProvidePubFollowService(getter pub.IDGetterService, req pub.RequesterService) *PubFollowService {
	return &PubFollowService{getter: getter, req: req}
}

var (
	ErrUserInboxNotSet  = errors.New("user inbox not set or valid")
	ErrUserInboxInvalid = errors.New("user inbox is invalid")
)

func (s *PubFollowService) SendFollowRequest(reqID *url.URL, follower, following *db.User) error {
	inboxURL, err := s.getter.GetUserID(following, "inbox")
	if err != nil {
		return ErrUserInboxInvalid
	}
	follow, err := s.createFollowRequest(reqID, follower, following)
	if err != nil {
		return err
	}
	return s.req.PostToInbox(inboxURL, follow)
}

func (s *PubFollowService) createFollowRequest(reqID *url.URL, follower, following *db.User) (vocab.ActivityStreamsFollow, error) {
	follow := streams.NewActivityStreamsFollow()

	followID := streams.NewJSONLDIdProperty()
	followID.Set(reqID)
	follow.SetJSONLDId(followID)

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
	inboxURL, err := s.getter.GetUserID(&req.Follower, "inbox")
	if err != nil {
		return ErrUserInboxInvalid
	}
	accept, err := s.createAccept(req)
	if err != nil {
		return err
	}
	return s.req.PostToInbox(inboxURL, accept)
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

package users

import (
	"errors"
	"fmt"
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
	return s.req.PostToInbox(inboxURL, follow, follower)
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

type parsedUndoFollow struct {
	RequestID *url.URL
	ActorURI  *url.URL
	ObjectURI *url.URL
}

func (s *PubFollowService) ProcessUndo(reject vocab.ActivityStreamsUndo) (parsedUndoFollow, error) {
	var activityActor *url.URL
	if reject.GetActivityStreamsActor() != nil && reject.GetActivityStreamsActor().Len() == 1 && reject.GetActivityStreamsActor().At(0).IsIRI() {
		activityActor = reject.GetActivityStreamsActor().At(0).GetIRI()
	}

	if reject.GetActivityStreamsObject().Len() != 1 {
		return parsedUndoFollow{}, errors.New("invalid reject request: not support multiple objects")
	}
	if !reject.GetActivityStreamsObject().At(0).IsActivityStreamsFollow() {
		return parsedUndoFollow{}, errors.New("invalid reject request: object is not a follow activity")
	}
	object := reject.GetActivityStreamsObject().At(0).GetActivityStreamsFollow()
	if object == nil {
		return parsedUndoFollow{}, errors.New("invalid reject request: object is nil")
	}

	var parsed parsedUndoFollow
	if object.GetJSONLDId() != nil && object.GetJSONLDId().IsIRI() {
		reqID := object.GetJSONLDId().GetIRI()
		parsed.RequestID = reqID
	}
	if object.GetActivityStreamsActor() != nil && object.GetActivityStreamsActor().Len() == 1 && object.GetActivityStreamsActor().At(0).IsIRI() {
		actorURI := object.GetActivityStreamsActor().At(0).GetIRI()
		parsed.ActorURI = actorURI
	}
	if object.GetActivityStreamsObject() != nil && object.GetActivityStreamsObject().Len() == 1 && object.GetActivityStreamsObject().At(0).IsIRI() {
		objectURI := object.GetActivityStreamsObject().At(0).GetIRI()
		parsed.ObjectURI = objectURI
	}

	if *parsed.ActorURI != *activityActor {
		return parsed, errors.New("invalid reject request: actor URI does not match the actor of the reject activity")
	}

	if parsed.ActorURI == nil || parsed.ObjectURI == nil {
		return parsed, errors.New("invalid reject request")
	}

	return parsed, nil
}

type parsedAcceptRequest struct {
	ReqID     *url.URL
	ActorURI  *url.URL
	ObjectURI *url.URL
}

type parsedRejectRequest struct {
	ActorURI  *url.URL
	ObjectURI *url.URL
}

func (s *PubFollowService) ProcessReject(reject vocab.ActivityStreamsReject) (parsedRejectRequest, error) {
	var activityActor *url.URL
	if reject.GetActivityStreamsActor() != nil && reject.GetActivityStreamsActor().Len() == 1 && reject.GetActivityStreamsActor().At(0).IsIRI() {
		activityActor = reject.GetActivityStreamsActor().At(0).GetIRI()
	}

	if reject.GetActivityStreamsObject().Len() != 1 {
		return parsedRejectRequest{}, errors.New("invalid reject request: not support multiple objects")
	}
	if !reject.GetActivityStreamsObject().At(0).IsActivityStreamsFollow() {
		return parsedRejectRequest{}, errors.New("invalid reject request: object is not a follow activity")
	}
	object := reject.GetActivityStreamsObject().At(0).GetActivityStreamsFollow()
	if object == nil {
		return parsedRejectRequest{}, errors.New("invalid reject request: object is nil")
	}

	var parsed parsedRejectRequest
	if object.GetActivityStreamsActor() != nil && object.GetActivityStreamsActor().Len() == 1 && object.GetActivityStreamsActor().At(0).IsIRI() {
		actorURI := object.GetActivityStreamsActor().At(0).GetIRI()
		parsed.ActorURI = actorURI
	}
	if object.GetActivityStreamsObject() != nil && object.GetActivityStreamsObject().Len() == 1 && object.GetActivityStreamsObject().At(0).IsIRI() {
		objectURI := object.GetActivityStreamsObject().At(0).GetIRI()
		parsed.ObjectURI = objectURI
	}

	if (*parsed.ActorURI != *activityActor) && (*parsed.ObjectURI != *activityActor) {
		return parsed, errors.New("invalid reject request: actor URI does not match the actor of the reject activity")
	}

	if parsed.ActorURI == nil || parsed.ObjectURI == nil {
		return parsed, errors.New("invalid reject request")
	}

	return parsed, nil
}

func (s *PubFollowService) ProcessAccept(accept vocab.ActivityStreamsAccept) (parsedAcceptRequest, error) {
	var activityActor *url.URL
	if accept.GetActivityStreamsActor() != nil && accept.GetActivityStreamsActor().Len() == 1 && accept.GetActivityStreamsActor().At(0).IsIRI() {
		activityActor = accept.GetActivityStreamsActor().At(0).GetIRI()
	}

	if accept.GetActivityStreamsObject().Len() != 1 {
		return parsedAcceptRequest{}, errors.New("invalid accept request: not support multiple objects")
	}
	if !accept.GetActivityStreamsObject().At(0).IsActivityStreamsFollow() {
		return parsedAcceptRequest{}, errors.New("invalid accept request: object is not a follow activity")
	}
	object := accept.GetActivityStreamsObject().At(0).GetActivityStreamsFollow()
	if object == nil {
		return parsedAcceptRequest{}, errors.New("invalid accept request: object is nil")
	}

	var parsed parsedAcceptRequest
	if object.GetJSONLDId() != nil && object.GetJSONLDId().IsIRI() {
		reqID := object.GetJSONLDId().GetIRI()
		parsed.ReqID = reqID
	}
	if object.GetActivityStreamsActor() != nil && object.GetActivityStreamsActor().Len() == 1 && object.GetActivityStreamsActor().At(0).IsIRI() {
		actorURI := object.GetActivityStreamsActor().At(0).GetIRI()
		parsed.ActorURI = actorURI
	}
	if object.GetActivityStreamsObject() != nil && object.GetActivityStreamsObject().Len() == 1 && object.GetActivityStreamsObject().At(0).IsIRI() {
		objectURI := object.GetActivityStreamsObject().At(0).GetIRI()
		parsed.ObjectURI = objectURI
	}

	if parsed.ActorURI != nil && ((*parsed.ActorURI != *activityActor) && (*parsed.ObjectURI != *activityActor)) {
		return parsed, errors.New("invalid accept request: actor URI does not match the actor of the accept activity")
	}

	if parsed.ReqID == nil && (parsed.ActorURI == nil || parsed.ObjectURI == nil) {
		return parsed, errors.New("invalid accept request")
	}

	return parsed, nil
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
	return s.req.PostToInbox(inboxURL, accept, &req.Followee)
}

func (s *PubFollowService) createAccept(req *db.UserFollowRequest) (vocab.ActivityStreamsAccept, error) {
	if !req.Incoming {
		return nil, fmt.Errorf("accepting an outgoing request is not possible")
	}

	accept := streams.NewActivityStreamsAccept()

	reqID, err := s.getter.GetFollowRequestID(req)
	if err != nil {
		return nil, err
	}
	objectProp := streams.NewActivityStreamsObjectProperty()
	objectProp.AppendIRI(reqID)
	accept.SetActivityStreamsObject(objectProp)

	actorURI, err := s.getter.GetUserID(&req.Followee, "")
	if err != nil {
		return nil, err
	}
	actorProp := streams.NewActivityStreamsActorProperty()
	actorProp.AppendIRI(actorURI)
	accept.SetActivityStreamsActor(actorProp)

	return accept, nil
}

type rejectUnfollowRequest struct {
	Follower *db.User
	Followee *db.User
}

func (s *PubFollowService) SendUnfollowRequest(req rejectUnfollowRequest) error {
	// prerequisite: req.Follower is local and req.Followee is remote
	if req.Follower.Host.Valid {
		return fmt.Errorf("follower is not a local user")
	}
	if !req.Followee.Host.Valid {
		return fmt.Errorf("followee is not a remote user")
	}

	var inboxURL *url.URL
	if req.Followee.Inbox.Valid {
		var err error
		inboxURL, err = url.Parse(req.Followee.Inbox.String)
		if err != nil {
			return fmt.Errorf("invalid inbox URL: %w", err)
		}
	} else {
		return ErrUserInboxNotSet
	}

	unfollow := streams.NewActivityStreamsUndo()

	actorID := streams.NewActivityStreamsActorProperty()
	actorURI, err := s.getter.GetUserID(req.Follower, "")
	if err != nil {
		return err
	}
	actorID.AppendIRI(actorURI)
	unfollow.SetActivityStreamsActor(actorID)

	object := streams.NewActivityStreamsObjectProperty()

	followObj := streams.NewActivityStreamsFollow()

	followActor := streams.NewActivityStreamsActorProperty()
	followActor.AppendIRI(actorURI)
	followObj.SetActivityStreamsActor(followActor)

	followObject := streams.NewActivityStreamsObjectProperty()
	objectURL, err := url.Parse(req.Followee.URI.String)
	if err != nil {
		return err
	}
	followObject.AppendIRI(objectURL)
	followObj.SetActivityStreamsObject(followObject)

	object.AppendActivityStreamsFollow(followObj)

	unfollow.SetActivityStreamsObject(object)

	return s.req.PostToInbox(inboxURL, unfollow, req.Follower)
}

type parsedFollowRequest struct {
	RequestID *url.URL
	ActorURI  *url.URL
	ObjectURI *url.URL
}

func (s *PubFollowService) ProcessFollow(follow vocab.ActivityStreamsFollow) (parsedFollowRequest, error) {
	var activityActor *url.URL
	if follow.GetActivityStreamsActor() != nil && follow.GetActivityStreamsActor().Len() == 1 && follow.GetActivityStreamsActor().At(0).IsIRI() {
		activityActor = follow.GetActivityStreamsActor().At(0).GetIRI()
	}

	if follow.GetActivityStreamsObject().Len() != 1 {
		return parsedFollowRequest{}, errors.New("invalid follow request: not support multiple objects")
	}
	if !follow.GetActivityStreamsObject().At(0).IsIRI() {
		return parsedFollowRequest{}, errors.New("invalid follow request: object is not an IRI")
	}
	object := follow.GetActivityStreamsObject().At(0).GetIRI()
	if object == nil {
		return parsedFollowRequest{}, errors.New("invalid follow request: object is nil")
	}

	var parsed parsedFollowRequest
	if follow.GetJSONLDId() != nil && follow.GetJSONLDId().IsIRI() {
		reqID := follow.GetJSONLDId().GetIRI()
		parsed.RequestID = reqID
	}
	if follow.GetActivityStreamsActor() != nil && follow.GetActivityStreamsActor().Len() == 1 && follow.GetActivityStreamsActor().At(0).IsIRI() {
		actorURI := follow.GetActivityStreamsActor().At(0).GetIRI()
		parsed.ActorURI = actorURI
	}
	if follow.GetActivityStreamsObject() != nil && follow.GetActivityStreamsObject().Len() == 1 && follow.GetActivityStreamsObject().At(0).IsIRI() {
		objectURI := follow.GetActivityStreamsObject().At(0).GetIRI()
		parsed.ObjectURI = objectURI
	}

	if *parsed.ActorURI != *activityActor {
		return parsed, errors.New("invalid follow request: actor URI does not match the actor of the follow activity")
	}

	if parsed.RequestID == nil || parsed.ActorURI == nil || parsed.ObjectURI == nil {
		return parsed, errors.New("invalid follow request")
	}

	return parsed, nil
}

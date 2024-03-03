package users

import (
	"net/url"

	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/pub"
)

type PubUserService struct {
	create    UserCreateService
	requester pub.RequesterService
	webfinger *pub.WebfingerService
}

func ProvidePubUserService(create UserCreateService, req pub.RequesterService, webFinger *pub.WebfingerService) *PubUserService {
	return &PubUserService{create: create, requester: req, webfinger: webFinger}
}

func (s *PubUserService) FetchRemoteUserByUsername(username string, host string) (*db.User, error) {
	uri, err := s.webfinger.FetchUserURI(username, host)
	if err != nil {
		return nil, err
	}
	return s.FetchRemoteUser(uri.API)
}

func (s *PubUserService) FetchRemoteUser(uri *url.URL) (*db.User, error) {
	fetched, err := s.requester.FetchUser(uri)
	if err != nil {
		return nil, err
	}
	host := uri.Hostname()

	// save the user
	remoteUser := translatePerson(fetched)
	remoteUser.Host = host
	user, err := s.create.UpdateRemoteUser(remoteUser)
	if err != nil {
		return nil, err
	}

	return user, nil
}

func translatePerson(person vocab.ActivityStreamsPerson) RemoteUser {
	inbox := ""
	if person.GetActivityStreamsInbox() != nil && person.GetActivityStreamsInbox().IsIRI() {
		inbox = person.GetActivityStreamsInbox().GetIRI().String()
	}

	outbox := ""
	if person.GetActivityStreamsOutbox() != nil && person.GetActivityStreamsOutbox().IsIRI() {
		outbox = person.GetActivityStreamsOutbox().GetIRI().String()
	}

	sharedInbox := ""
	if person.GetActivityStreamsSharedInbox() != nil && person.GetActivityStreamsSharedInbox().IsIRI() {
		sharedInbox = person.GetActivityStreamsSharedInbox().GetIRI().String()
	}

	following := ""
	if person.GetActivityStreamsFollowing() != nil && person.GetActivityStreamsFollowing().IsIRI() {
		following = person.GetActivityStreamsFollowing().GetIRI().String()
	}

	followers := ""
	if person.GetActivityStreamsFollowers() != nil && person.GetActivityStreamsFollowers().IsIRI() {
		followers = person.GetActivityStreamsFollowers().GetIRI().String()
	}

	liked := ""
	if person.GetActivityStreamsLiked() != nil && person.GetActivityStreamsLiked().IsIRI() {
		liked = person.GetActivityStreamsLiked().GetIRI().String()
	}

	preferredUsername := ""
	if person.GetActivityStreamsPreferredUsername() != nil && person.GetActivityStreamsPreferredUsername().IsXMLSchemaString() {
		preferredUsername = person.GetActivityStreamsPreferredUsername().GetXMLSchemaString()
	}

	name := ""
	if person.GetActivityStreamsName() != nil && person.GetActivityStreamsName().Len() > 0 && person.GetActivityStreamsName().At(0).IsXMLSchemaString() {
		name = person.GetActivityStreamsName().At(0).GetXMLSchemaString()
	}

	publicKeys := make([]RemoteUserKey, 0)
	if person.GetW3IDSecurityV1PublicKey() != nil {
		p := person.GetW3IDSecurityV1PublicKey()
		iter := p.Begin()
		for iter != nil {
			k := iter.Get()

			keyID := k.GetJSONLDId().Get().String()
			publicKey := k.GetW3IDSecurityV1PublicKeyPem().Get()

			publicKeys = append(publicKeys, RemoteUserKey{
				ID:           keyID,
				PublicKeyPem: publicKey})

			iter = iter.Next()
		}
	}

	return RemoteUser{
		ID:                person.GetJSONLDId().Get().String(),
		PreferredUsername: preferredUsername,
		Name:              name,
		Inbox:             inbox,
		Outbox:            outbox,
		SharedInbox:       sharedInbox,
		Following:         following,
		Followers:         followers,
		Liked:             liked,
		Keys:              publicKeys,
	}
}

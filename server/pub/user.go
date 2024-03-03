package pub

import (
	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
)

type PubUserService struct {
	getter IDGetterService
}

func ProvidePubUserService(getter IDGetterService) *PubUserService {
	return &PubUserService{getter: getter}
}

func (s *PubUserService) CreateUserObject(user *db.User) (vocab.ActivityStreamsPerson, error) {
	userURI, err := s.getter.GetUserID(user, "")
	if err != nil {
		return nil, err
	}
	inboxURI, err := s.getter.GetUserID(user, "inbox")
	if err != nil {
		return nil, err
	}
	outboxURI, err := s.getter.GetUserID(user, "outbox")
	if err != nil {
		return nil, err
	}
	followingURI, err := s.getter.GetUserID(user, "following")
	if err != nil {
		return nil, err
	}
	followersURI, err := s.getter.GetUserID(user, "followers")
	if err != nil {
		return nil, err
	}
	likedURI, err := s.getter.GetUserID(user, "liked")
	if err != nil {
		return nil, err
	}

	actor := streams.NewActivityStreamsPerson()

	actorId := streams.NewJSONLDIdProperty()
	actorId.Set(userURI)

	inbox := streams.NewActivityStreamsInboxProperty()
	inbox.SetIRI(inboxURI)

	outbox := streams.NewActivityStreamsOutboxProperty()
	outbox.SetIRI(outboxURI)

	following := streams.NewActivityStreamsFollowingProperty()
	following.SetIRI(followingURI)

	followers := streams.NewActivityStreamsFollowersProperty()
	followers.SetIRI(followersURI)

	liked := streams.NewActivityStreamsLikedProperty()
	liked.SetIRI(likedURI)

	name := streams.NewActivityStreamsNameProperty()
	name.AppendXMLSchemaString(user.Nickname)

	preferredUsername := streams.NewActivityStreamsPreferredUsernameProperty()
	preferredUsername.SetXMLSchemaString(user.Username)

	actor.SetJSONLDId(actorId)
	actor.SetActivityStreamsInbox(inbox)
	actor.SetActivityStreamsOutbox(outbox)
	actor.SetActivityStreamsFollowing(following)
	actor.SetActivityStreamsFollowers(followers)
	actor.SetActivityStreamsLiked(liked)
	actor.SetActivityStreamsName(name)
	actor.SetActivityStreamsPreferredUsername(preferredUsername)

	if user.PublicKey.Valid {
		keyID, err := s.getter.GetUserID(user, "publicKey")
		if err != nil {
			return nil, err
		}

		publicKeys := streams.NewW3IDSecurityV1PublicKeyProperty()

		publicKey := streams.NewW3IDSecurityV1PublicKey()

		keyId := streams.NewJSONLDIdProperty()
		keyId.SetIRI(keyID)
		publicKey.SetJSONLDId(keyId)

		keyOwner := streams.NewW3IDSecurityV1OwnerProperty()
		keyOwner.SetIRI(userURI)
		publicKey.SetW3IDSecurityV1Owner(keyOwner)

		publicKeyPem := streams.NewW3IDSecurityV1PublicKeyPemProperty()
		publicKeyPem.Set(user.PublicKey.String)
		publicKey.SetW3IDSecurityV1PublicKeyPem(publicKeyPem)

		publicKeys.AppendW3IDSecurityV1PublicKey(publicKey)
		actor.SetW3IDSecurityV1PublicKey(publicKeys)
	}

	return actor, nil
}

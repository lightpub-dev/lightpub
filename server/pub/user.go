package pub

import (
	"github.com/go-fed/activity/streams"
	"github.com/lightpub-dev/lightpub/db"
)

type PubUserService struct {
	getter IDGetterService
}

func ProvidePubUserService(getter IDGetterService) *PubUserService {
	return &PubUserService{getter: getter}
}

func (s *PubUserService) CreateUserObject(user *db.User) error {
	userURI, err := s.getter.GetUserID(user, "")
	if err != nil {
		return err
	}

	actor := streams.NewActivityStreamsActorProperty()
	actor.AppendIRI(userURI)
	return nil
}

package pub

import (
	"log"
	"net/url"

	"github.com/go-fed/activity/streams/vocab"
	"github.com/google/wire"
)

type RequesterService interface {
	PostToInbox(inboxURL *url.URL, activity interface{}) error
	FetchUser(uri *url.URL) (vocab.ActivityStreamsPerson, error)
}

var (
	GoRequesterServices = wire.NewSet(
		ProvideGoRequesterService,
		wire.Bind(new(RequesterService), new(*GoRequesterService)),
	)
)

type GoRequesterService struct{}

func ProvideGoRequesterService() *GoRequesterService {
	return &GoRequesterService{}
}

func (s *GoRequesterService) PostToInbox(inboxURL *url.URL, activity interface{}) error {
	log.Printf("Sending to %s: %v", inboxURL, activity)
	return nil
}

func (s *GoRequesterService) FetchUser(uri *url.URL) (vocab.ActivityStreamsPerson, error) {
	log.Fatalf("fetch user not implemented")
	return nil, nil
}

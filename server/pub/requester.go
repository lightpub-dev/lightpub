package pub

import (
	"log"
	"net/url"

	"github.com/google/wire"
)

type RequesterService interface {
	PostToInbox(inboxURL *url.URL, activity interface{}) error
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

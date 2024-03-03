package pub

import (
	"log"
	"net/url"
)

type WebfingerService struct {
}

func ProvideWebfingerService() *WebfingerService {
	return &WebfingerService{}
}

type WebfingerUser struct {
	API *url.URL
}

func (s *WebfingerService) FetchUserURI(username, host string) (WebfingerUser, error) {
	log.Fatalf("webfinger not implemented")
	return WebfingerUser{API: &url.URL{}}, nil
}

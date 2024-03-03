package pub

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"time"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/google/wire"
)

const (
	ActivityJsonType = "application/activity+json"
)

type RequesterService interface {
	PostToInbox(inboxURL *url.URL, activity interface{}) error
	FetchUser(uri *url.URL) (vocab.ActivityStreamsPerson, error)
	FetchWebfinger(username, host string) ([]byte, error)
}

var (
	GoRequesterServices = wire.NewSet(
		ProvideGoRequesterService,
		wire.Bind(new(RequesterService), new(*GoRequesterService)),
	)
)

type GoRequesterOptions struct {
	Timeout time.Duration
}

type GoRequesterService struct {
	client  *http.Client
	options GoRequesterOptions
}

func ProvideGoRequesterService(client *http.Client, options GoRequesterOptions) *GoRequesterService {
	return &GoRequesterService{client, options}
}

func (s *GoRequesterService) makeContext() context.Context {
	ctx, _ := context.WithTimeout(context.Background(), s.options.Timeout)
	return ctx

}

func (s *GoRequesterService) PostToInbox(inboxURL *url.URL, activity interface{}) error {
	log.Printf("Sending to %s: %v", inboxURL, activity)
	return nil
}

func (s *GoRequesterService) FetchUser(uri *url.URL) (vocab.ActivityStreamsPerson, error) {
	req, err := http.NewRequestWithContext(s.makeContext(), "GET", uri.String(), nil)
	if err != nil {
		return nil, err
	}
	req.Header.Add("accept", ActivityJsonType)
	resp, err := s.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	var bodyJson map[string]interface{}
	if err := json.Unmarshal(body, &bodyJson); err != nil {
		return nil, err
	}

	var person vocab.ActivityStreamsPerson
	reader := func(c context.Context, p vocab.ActivityStreamsPerson) error {
		person = p
		return nil
	}

	resolver, err := streams.NewJSONResolver(reader)
	if err != nil {
		return nil, err
	}
	err = resolver.Resolve(s.makeContext(), bodyJson)
	if err != nil {
		return nil, err
	}
	return person, nil
}

func (s *GoRequesterService) FetchWebfinger(username, host string) ([]byte, error) {
	u := url.URL{
		Scheme: "https",
		Host:   host,
		Path:   "/.well-known/webfinger",
	}
	q := u.Query()
	q.Add("resource", fmt.Sprintf("acct:%s@%s", username, host))
	u.RawQuery = q.Encode()
	log.Printf("Fetching webfinger: %s", u.String())
	req, err := http.NewRequestWithContext(s.makeContext(), http.MethodGet, u.String(), nil)
	if err != nil {
		return nil, err
	}
	resp, err := s.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	return io.ReadAll(resp.Body)
}

package pub

import (
	"bytes"
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
	"github.com/lightpub-dev/lightpub/db"
)

const (
	ActivityJsonType = "application/activity+json"
)

type RequesterService interface {
	PostToInbox(inboxURL *url.URL, activity vocab.Type, actor *db.User) error
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
	client    *http.Client
	options   GoRequesterOptions
	signature *SignatureService
}

func ProvideGoRequesterService(client *http.Client, options GoRequesterOptions, signature *SignatureService) *GoRequesterService {
	return &GoRequesterService{client, options, signature}
}

func (s *GoRequesterService) makeContext() context.Context {
	ctx, _ := context.WithTimeout(context.Background(), s.options.Timeout)
	return ctx
}

func (s *GoRequesterService) PostToInbox(inboxURL *url.URL, activity vocab.Type, actor *db.User) error {
	jsonMap, err := activity.Serialize()
	if err != nil {
		return err
	}
	// jsonMap["@context"] = "https://www.w3.org/ns/activitystreams"
	body, err := json.Marshal(jsonMap)
	if err != nil {
		return err
	}
	bodyBuf := bytes.NewBuffer(body)

	req, err := http.NewRequestWithContext(s.makeContext(), "POST", inboxURL.String(), nil)
	if err != nil {
		return err
	}
	req.Header.Add("accept", ActivityJsonType)
	req.Header.Add("content-type", ActivityJsonType)
	req.Body = io.NopCloser(bodyBuf)

	// sign the request
	if err := s.signature.Sign(actor, req, body); err != nil {
		return err
	}

	resp, err := s.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	// show body if the response is not 2xx
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return err
		}
		log.Printf("PostToInbox: status=%d %s", resp.StatusCode, body)
		return fmt.Errorf("PostToInbox: status=%d %s", resp.StatusCode, body)
	}

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

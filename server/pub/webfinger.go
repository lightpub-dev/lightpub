package pub

import (
	"encoding/json"
	"log"
	"net/url"
)

type WebfingerService struct {
	req RequesterService
}

func ProvideWebfingerService(req RequesterService) *WebfingerService {
	return &WebfingerService{req}
}

type WebfingerUser struct {
	API *url.URL
}

// Define a struct for the link objects within the "links" array
type link struct {
	Rel  string `json:"rel"`  // Relationship type
	Type string `json:"type"` // MIME type
	Href string `json:"href"` // URL
}

// Define the main struct to hold the entire JSON object
type webFingerResponse struct {
	Subject string `json:"subject"` // Subject identifier
	Links   []link `json:"links"`   // Array of Link objects
}

func (s *WebfingerService) FetchUserURI(username, host string) (WebfingerUser, error) {
	body, err := s.req.FetchWebfinger(username, host)
	if err != nil {
		log.Printf("Error fetching webfinger: %s", err)
		return WebfingerUser{}, err
	}

	// Parse the JSON response
	var wfResp webFingerResponse
	err = json.Unmarshal(body, &wfResp)
	if err != nil {
		log.Printf("Error parsing webfinger response: %s", err)
		return WebfingerUser{}, err
	}

	// Find the "self" link
	var selfLink link
	for _, l := range wfResp.Links {
		if l.Rel == "self" && l.Type == "application/activity+json" {
			selfLink = l
			break
		}
	}

	// Parse the "self" link into a URL
	apiURL, err := url.Parse(selfLink.Href)
	if err != nil {
		log.Printf("Error parsing API URL: %s", err)
		return WebfingerUser{}, err
	}

	return WebfingerUser{apiURL}, nil
}

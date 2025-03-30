/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

package apub

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
)

func (s *Requester) fetchUserURLByWebfinger(ctx context.Context, username string, domain string) (*url.URL, error) {
	if domain == "" {
		return nil, fmt.Errorf("domain is empty")
	}
	if domain == s.baseURL.Host {
		return nil, fmt.Errorf("domain is my own")
	}

	// Construct the WebFinger URL
	u := &url.URL{
		Scheme: "https",
		Host:   domain,
		Path:   "/.well-known/webfinger",
	}
	q := url.Values{}
	q.Add("resource", fmt.Sprintf("acct:%s@%s", username, domain))
	u.RawQuery = q.Encode()
	webfingerURL := u.String()

	// Create a new HTTP request
	req, err := http.NewRequestWithContext(ctx, "GET", webfingerURL, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create WebFinger request: %w", err)
	}

	// Set Accept header to request JRD format
	req.Header.Set("Accept", "application/jrd+json, application/json")

	// Perform the HTTP request
	resp, err := s.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("WebFinger request failed: %w", err)
	}
	defer resp.Body.Close()

	// Check if the response status is OK
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("WebFinger request returned status: %d", resp.StatusCode)
	}

	// Parse the JSON response
	var webfingerResp struct {
		Subject string `json:"subject"`
		Links   []struct {
			Rel  string `json:"rel"`
			Type string `json:"type"`
			Href string `json:"href"`
		} `json:"links"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&webfingerResp); err != nil {
		return nil, fmt.Errorf("failed to decode WebFinger response: %w", err)
	}

	// Look for the self link with ActivityPub type
	for _, link := range webfingerResp.Links {
		if link.Rel == "self" && (link.Type == ApubActivityJsonType ||
			link.Type == ApubLdJsonType) {
			// Parse the URL
			parsedURL, err := url.Parse(link.Href)
			if err != nil {
				return nil, fmt.Errorf("failed to parse WebFinger URL: %w", err)
			}
			if parsedURL.Scheme != "https" && parsedURL.Scheme != "http" {
				return nil, fmt.Errorf("invalid UserID URL scheme: %s", parsedURL.Scheme)
			}
			return parsedURL, nil
		}
	}

	return nil, fmt.Errorf("no ActivityPub profile URL found in WebFinger response")
}

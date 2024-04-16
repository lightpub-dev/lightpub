package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

type Requester struct {
	baseURL string
	client  *http.Client
}

func NewRequester(baseURL string, client *http.Client) *Requester {
	return &Requester{
		baseURL,
		client,
	}
}

func (r *Requester) Post(url string, jsonBody interface{}) (*http.Response, error) {
	// convert to json
	jsonBytes, err := json.Marshal(jsonBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal json: %w", err)
	}

	// Do the request
	resp, err := r.client.Post(r.baseURL+url, "application/json", bytes.NewBuffer(jsonBytes))
	if err != nil {
		return nil, fmt.Errorf("failed to do request: %w", err)
	}

	return resp, nil
}

package apub

import (
	"strings"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

type CreateActivity struct {
	ID     string          `json:"id"`
	Kind   string          `json:"type"`
	Actor  URI             `json:"actor"`
	To     []string        `json:"to"`
	Cc     []string        `json:"cc"`
	Object CreatableObject `json:"object"`
}

type CreatableObject struct {
}

type NoteObject struct {
	ID           URI              `json:"id"`
	Kind         string           `json:"type"`
	AttributedTo URI              `json:"attributedTo"`
	Content      string           `json:"content"`
	Published    time.Time        `json:"published"`
	Updated      *time.Time       `json:"updated,omitempty"`
	To           []string         `json:"to"`
	Cc           []string         `json:"cc"`
	URL          *string          `json:"url,omitempty"`
	Source       *NoteSource      `json:"source,omitempty"`
	InReplyTo    *URI             `json:"inReplyTo,omitempty"`
	Sensitive    *bool            `json:"sensitive,omitempty"`
	Tag          []NoteTag        `json:"tag,omitempty"`
	Attachment   []NoteAttachment `json:"attachment,omitempty"`
}

type NoteSource struct {
	Content   string `json:"content"`
	MediaType string `json:"mediaType"`
}

type NoteTag struct {
	Kind string  `json:"type"`
	Name *string `json:"name,omitempty"`
	Href *URI    `json:"href,omitempty"`
}

type NoteAttachment struct {
	Kind      string `json:"type"`
	URL       URI    `json:"url"`
	MediaType string `json:"mediaType"`
}

func (n *NoteObject) InferredVisibility() types.NoteVisibility {
	if containsPublicURL(n.To) {
		return types.NoteVisibilityPublic
	} else if containsPublicURL(n.Cc) {
		return types.NoteVisibilityUnlisted
	}

	for _, t := range n.To {
		if strings.HasSuffix(t, followersSuffix) {
			return types.NoteVisibilityFollower
		}
	}

	return types.NoteVisibilityPrivate
}

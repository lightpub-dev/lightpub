package apub

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

type CreatableObjectType string

const (
	CreatableObjectTypeNote CreatableObjectType = "Note"
)

type CreateActivity struct {
	ID     string          `json:"id" validate:"required"`
	Kind   string          `json:"type" validate:"required"`
	Actor  URI             `json:"actor" validate:"required"`
	To     []string        `json:"to"`
	Cc     []string        `json:"cc"`
	Object CreatableObject `json:"object" validate:"required"`
}

type CreatableObject struct {
	Kind CreatableObjectType

	NoteObject *NoteObject
}

func (c CreatableObject) MarshalJSON() ([]byte, error) {
	switch c.Kind {
	case CreatableObjectTypeNote:
		return json.Marshal(*c.NoteObject)
	}

	return nil, fmt.Errorf("unknown creatable object type: %s", c.Kind)
}

func (c *CreatableObject) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Note":
		c.Kind = CreatableObjectTypeNote
		var n NoteObject
		if err := json.Unmarshal(data, &n); err != nil {
			return fmt.Errorf("error unmarshalling note object: %w", err)
		}
		if err := validate.Struct(n); err != nil {
			return fmt.Errorf("error validating note object: %w", err)
		}
		c.NoteObject = &n
	default:
		return fmt.Errorf("unknown creatable object Type: %s", typ)
	}

	return nil
}

type NoteObject struct {
	ID           URI              `json:"id" validate:"required"`
	Kind         string           `json:"type" validate:"required"`
	AttributedTo URI              `json:"attributedTo" validate:"required"`
	Content      string           `json:"content" validate:"required"`
	Published    time.Time        `json:"published" validate:"required"`
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
	return inferVisibility(n.To, n.Cc)
}

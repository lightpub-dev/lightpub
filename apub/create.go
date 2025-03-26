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
	ID     URI             `json:"id" validate:"required,http_url"`
	Kind   string          `json:"type" validate:"required,eq=Create"`
	Actor  URI             `json:"actor" validate:"required,http_url"`
	To     []string        `json:"to" validate:"dive,http_url"`
	Cc     []string        `json:"cc" validate:"dive,http_url"`
	Object CreatableObject `json:"object" validate:"required"`
}

func NewCreateActivity(object CreatableObject) CreateActivity {
	return CreateActivity{
		ID:     activityIDFromObject("Create", object.ID()),
		Kind:   "Create",
		Actor:  object.Actor(),
		To:     object.To(),
		Cc:     object.Cc(),
		Object: object,
	}
}

type CreatableObject struct {
	Kind CreatableObjectType

	NoteObject *NoteObject
}

func (c CreatableObject) ID() URI {
	switch c.Kind {
	case CreatableObjectTypeNote:
		return c.NoteObject.ID
	}

	panic("unknown creatable object type")
}

func (c CreatableObject) Actor() URI {
	switch c.Kind {
	case CreatableObjectTypeNote:
		return c.NoteObject.AttributedTo
	}

	panic("unknown creatable object type")
}

func (c CreatableObject) To() []string {
	switch c.Kind {
	case CreatableObjectTypeNote:
		return c.NoteObject.To
	}

	panic("unknown creatable object type")
}

func (c CreatableObject) Cc() []string {
	switch c.Kind {
	case CreatableObjectTypeNote:
		return c.NoteObject.Cc
	}

	panic("unknown creatable object type")
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
	ID           URI              `json:"id" validate:"required,http_url"`
	Kind         string           `json:"type" validate:"required,oneof=Note Article"`
	AttributedTo URI              `json:"attributedTo" validate:"required,http_url"`
	Content      string           `json:"content" validate:"required"`
	Published    time.Time        `json:"published" validate:"required"`
	Updated      *time.Time       `json:"updated,omitempty"`
	To           []string         `json:"to"`
	Cc           []string         `json:"cc"`
	URL          string           `json:"url,omitempty" validate:"http_url"`
	Source       *NoteSource      `json:"source,omitempty"`
	InReplyTo    URI              `json:"inReplyTo,omitempty" validate:"http_url"`
	Sensitive    *bool            `json:"sensitive,omitempty"`
	Tag          []NoteTag        `json:"tag,omitempty"`
	Attachment   []NoteAttachment `json:"attachment,omitempty"`
}

type NoteSource struct {
	Content   string `json:"content" validate:"required"`
	MediaType string `json:"mediaType" validate:"required"`
}

type NoteTag struct {
	Kind string `json:"type" validate:"required"`
	Name string `json:"name,omitempty"`
	Href URI    `json:"href,omitempty" validate:"http_url"`
}

type NoteAttachment struct {
	Kind      string `json:"type" validate:"required"`
	URL       URI    `json:"url" validate:"required,http_url"`
	MediaType string `json:"mediaType" validate:"required"`
}

func (n *NoteObject) InferredVisibility() types.NoteVisibility {
	return inferVisibility(n.To, n.Cc)
}

func NewNoteObject(n *types.ApubNote) *NoteObject {
	tags := make([]NoteTag, 0, len(n.Apub.Hashtags)+len(n.Apub.Mentions))
	for _, hashtag := range n.Apub.Hashtags {
		tags = append(tags, NoteTag{
			Kind: "Hashtag",
			Name: hashtag.Name,
			Href: hashtag.TimelineURL,
		})
	}
	for _, mention := range n.Apub.Mentions {
		tags = append(tags, NoteTag{
			Kind: "Mention",
			Name: mention.Specifier,
			Href: mention.URL,
		})
	}

	attachments := make([]NoteAttachment, 0, len(n.Apub.Uploads))
	for _, upload := range n.Apub.Uploads {
		attachments = append(attachments, NoteAttachment{
			Kind:      "Document",
			URL:       upload.URL,
			MediaType: upload.MimeType,
		})
	}

	return &NoteObject{
		ID:           n.Apub.URL,
		Kind:         "Note",
		AttributedTo: n.Apub.AuthorURL,
		Content:      n.Basic.Content.Data,
		Published:    n.Basic.CreatedAt,
		Updated:      n.Basic.UpdatedAt,
		To:           n.Apub.To,
		Cc:           n.Apub.Cc,
		URL:          n.Apub.ViewURL,
		Source: &NoteSource{
			Content:   n.Basic.Content.Source,
			MediaType: n.Basic.Content.Type.MimeType(),
		},
		InReplyTo:  n.Apub.ReplyToURL,
		Sensitive:  &n.Basic.Sensitive,
		Tag:        tags,
		Attachment: attachments,
	}
}

func (n *NoteObject) AsCreatableObject() CreatableObject {
	return CreatableObject{
		Kind:       CreatableObjectTypeNote,
		NoteObject: n,
	}
}

package service

import (
	"bytes"
	"html"

	"github.com/lightpub-dev/lightpub/types"
	"github.com/yuin/goldmark"
)

func renderNoteContent(content string, contentType types.NoteContentType) (string, error) {
	switch contentType {
	case types.NoteContentTypePlain:
		return renderPlainText(content), nil
	case types.NoteContentTypeMD:
		return renderMarkdown(content)
	case types.NoteContentTypeHTML:
		return renderHtml(content), nil
	case types.NoteContentTypeLatex:
		// TODO: Implement LaTeX rendering
	}

	panic("unknown content type: " + contentType)
}

func renderPlainText(text string) string {
	return html.EscapeString(text)
}

func renderMarkdown(markdown string) (string, error) {
	var buf bytes.Buffer
	if err := goldmark.Convert([]byte(markdown), &buf); err != nil {
		return "", err
	}

	s := buf.String()
	return noteSanitizer.Sanitize(s), nil
}

func renderHtml(html string) string {
	return noteSanitizer.Sanitize(html)
}

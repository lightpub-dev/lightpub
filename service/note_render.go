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

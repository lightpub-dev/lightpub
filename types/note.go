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

package types

import (
	"html/template"
	"sort"
	"time"
)

const (
	NoteContentTypePlain NoteContentType = "plain"
	NoteContentTypeHTML  NoteContentType = "html"
	NoteContentTypeMD    NoteContentType = "md"
	NoteContentTypeLatex NoteContentType = "latex"

	NoteVisibilityPublic   NoteVisibility = "public"
	NoteVisibilityUnlisted NoteVisibility = "unlisted"
	NoteVisibilityFollower NoteVisibility = "follower"
	NoteVisibilityPrivate  NoteVisibility = "private"
)

var (
	NoteContentTypes = []NoteContentType{
		NoteContentTypePlain,
		NoteContentTypeHTML,
		NoteContentTypeMD,
		NoteContentTypeLatex,
	}

	Visibilities = []NoteVisibility{
		NoteVisibilityPublic,
		NoteVisibilityUnlisted,
		NoteVisibilityFollower,
		NoteVisibilityPrivate,
	}
)

func IsValidContentType(t string) bool {
	for _, tt := range NoteContentTypes {
		if tt == NoteContentType(t) {
			return true
		}
	}
	return false
}

func IsValidVisibility(v string) bool {
	for _, vv := range Visibilities {
		if vv == NoteVisibility(v) {
			return true
		}
	}
	return false
}

type NoteContentType string
type NoteVisibility string

func (n NoteVisibility) ValidAsRenote() bool {
	return n == NoteVisibilityPublic || n == NoteVisibilityUnlisted
}

func (n NoteVisibility) AcceptRenote() bool {
	return n == NoteVisibilityPublic || n == NoteVisibilityUnlisted
}

type SimpleNote struct {
	ID         NoteID
	Author     NoteAuthor
	Content    *NoteContent
	Visibility NoteVisibility
	CreatedAt  time.Time
	UpdatedAt  *time.Time

	ReplyToID  *NoteID
	RenoteOfID *NoteID
	DeletedAt  *time.Time

	Sensitive bool
	Uploads   []UploadID
}

func (n SimpleNote) Renotable() bool {
	return n.Visibility.AcceptRenote()
}

type NoteAuthor struct {
	ID       UserID
	Username string
	Nickname string
	Domain   string
}

func (n NoteAuthor) Specifier() string {
	return makeSpecifier(n.Username, n.Domain)
}

func (n NoteAuthor) IsLocal() bool {
	return n.Domain == EmptyDomain
}

func (n NoteAuthor) IsRemote() bool {
	return n.Domain != EmptyDomain
}

type NoteContent struct {
	Type NoteContentType
	Data string

	Source string
}

func (n NoteContent) RawHTML() template.HTML {
	return template.HTML(n.Data)
}

type DetailedNote struct {
	Basic   SimpleNote
	Details NoteDetails
}

type NoteDetails struct {
	ReplyCount    uint64
	RenoteCount   uint64
	ReactionCount map[string]uint64

	// viewer's interaction
	Renoted    bool
	Reacted    string
	Bookmarked bool
	IsMyNote   bool

	Hashtags []string
	Mentions []NoteMention

	RemoteViewURL string
}

func (n NoteDetails) ReactionList() []NoteReactionCount {
	list := make([]NoteReactionCount, 0, len(n.ReactionCount))
	for emoji, count := range n.ReactionCount {
		list = append(list, NoteReactionCount{Emoji: emoji, Count: count})
	}

	// sort by count desc, emoji asc
	sort.Slice(list, func(i, j int) bool {
		if list[i].Count == list[j].Count {
			return list[i].Emoji < list[j].Emoji
		}
		return list[i].Count > list[j].Count
	})

	return list
}

type NoteReactionCount struct {
	Emoji string
	Count uint64
}

type NoteMention struct {
	ID       UserID
	Username string
	Nickname string
	Domain   string
}

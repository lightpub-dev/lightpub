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
	"crypto"
	"html/template"
)

const (
	EmptyDomain = "" // Empty string means local server

	FollowStateNo      FollowState = 0
	FollowStateYes     FollowState = 1
	FollowStatePending FollowState = 2
)

type FollowState = int

type SimpleUser struct {
	ID       UserID
	Username string
	Domain   string // Empty string means local server
	Nickname string
	Bio      string
	Avatar   *UploadID
}

func makeSpecifier(username, domain string) string {
	if domain == EmptyDomain {
		return "@" + username
	}
	return "@" + username + "@" + domain
}

func (s SimpleUser) Specifier() string {
	return makeSpecifier(s.Username, s.Domain)
}

func (s SimpleUser) RawBio() template.HTML {
	return template.HTML(s.Bio)
}

func (s SimpleUser) IsRemote() bool {
	return s.Domain != EmptyDomain
}

func (s SimpleUser) IsLocal() bool {
	return s.Domain == EmptyDomain
}

type DetailedUser struct {
	Basic   SimpleUser
	Details DetailedUserModel
}

type DetailedUserModel struct {
	FollowCount      uint64
	FollowerCount    uint64
	NoteCount        uint64
	AutoFollowAccept bool
	HideFollows      bool
	RemoteURL        string
	RemoteViewURL    string

	// effective when the user is logged in
	IsFollowing FollowState
	IsFollowed  FollowState
	IsBlocking  bool
	IsBlocked   bool
	IsMe        bool
}

func (d DetailedUserModel) IsActuallyFollowed() bool {
	return d.IsFollowed == FollowStateYes
}

func (d DetailedUserModel) IsFollowRequested() bool {
	return d.IsFollowed == FollowStatePending
}

func (d DetailedUserModel) IsActuallyFollowing() bool {
	return d.IsFollowing == FollowStateYes
}

func (d DetailedUserModel) IsFollowRequesting() bool {
	return d.IsFollowing == FollowStatePending
}

func (d DetailedUserModel) CanFollow() bool {
	return d.IsFollowing == FollowStateNo && !d.IsMe
}

func (d DetailedUserModel) CanUnfollow() bool {
	return d.IsFollowing != FollowStateNo && !d.IsMe
}

func (d DetailedUserModel) CanRefuseFollow() bool {
	return d.IsFollowed != FollowStateNo && !d.IsMe
}

type ApubUser struct {
	Basic SimpleUser
	Apub  ApubUserData
}

func (a ApubUser) ID() string {
	return a.Apub.URL
}

func (a ApubUser) PublicKey() crypto.PublicKey {
	return a.Apub.PublicKey_
}

func (a ApubUser) PrivateKey() crypto.PrivateKey {
	return a.Apub.PrivateKey_
}

func (a ApubUser) KeyID() string {
	return a.Apub.KeyID_
}

func (a ApubUser) PreferredInbox() string {
	if a.Apub.SharedInbox != "" {
		return a.Apub.SharedInbox
	}
	return a.Apub.Inbox
}

type ApubUserData struct {
	PublicKey_  crypto.PublicKey
	PrivateKey_ crypto.PrivateKey
	KeyID_      string

	Bio string

	Inbox       string
	Outbox      string
	SharedInbox string // nullable

	ManuallyApprovesFollowers bool

	Following string // nullable
	Followers string // nullable
	URL       string
	ViewURL   string // nullable
}

func (a ApubUserData) ID() string {
	return a.URL
}

func (a ApubUserData) PublicKey() crypto.PublicKey {
	return a.PublicKey_
}

func (a ApubUserData) PrivateKey() crypto.PrivateKey {
	return a.PrivateKey_
}

func (a ApubUserData) KeyID() string {
	return a.KeyID_
}

func (a ApubUserData) PreferredInbox() string {
	if a.SharedInbox != "" {
		return a.SharedInbox
	}
	return a.Inbox
}

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
	"context"
	"fmt"

	"github.com/lightpub-dev/lightpub/apub"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
)

type calculateToAndCcResult struct {
	To      []string
	Cc      []string
	Inboxes []string
}

// calculateToAndCc calculates the to and cc fields for a local note.
// Passing a remote note will result in incorrect results.
func (s *State) calculateToAndCc(ctx context.Context, noteID types.NoteID, needInboxes bool) (calculateToAndCcResult, error) {
	// Fetch info from DB
	var note models.Note
	if err := s.DB(ctx).Unscoped().Where("notes.id = ? AND notes.deleted_at IS NULL AND notes.content IS NOT NULL", noteID).Joins("Author").First(&note).Error; err != nil {
		return calculateToAndCcResult{}, err
	}

	var mentions []models.NoteMention
	if err := s.DB(ctx).Where("note_id = ?", noteID).Joins("TargetUser").Find(&mentions).Error; err != nil {
		return calculateToAndCcResult{}, err
	}

	// Prepare results
	result := calculateToAndCcResult{}
	inboxAddedMap := make(map[string]struct{})
	addInbox := func(inbox string) {
		if !needInboxes {
			return
		}
		// 重複を省く
		if _, ok := inboxAddedMap[inbox]; ok {
			return
		}
		inboxAddedMap[inbox] = struct{}{}
		result.Inboxes = append(result.Inboxes, inbox)
	}

	// Reply
	if note.ReplyToID != nil {
		replyToUser, err := s.FindApubUserByID(ctx, *note.ReplyToID)
		if err != nil {
			return calculateToAndCcResult{}, fmt.Errorf("error fetching reply-to user %s: %w", *note.ReplyToID, err)
		}
		if replyToUser == nil {
			return calculateToAndCcResult{}, fmt.Errorf("reply-to user not found: %s", *note.ReplyToID)
		}

		result.To = append(result.To, replyToUser.Apub.URL)
		addInbox(replyToUser.Apub.PreferredInbox())
	}

	// Public URLs
	switch note.Visibility {
	case types.NoteVisibilityPublic:
		result.To = append(result.To, apub.PublicURL)
	case types.NoteVisibilityUnlisted:
		result.Cc = append(result.Cc, apub.PublicURL)
	}

	// Followers
	deliverToFollowers := false
	switch note.Visibility {
	case types.NoteVisibilityPublic:
		fallthrough
	case types.NoteVisibilityUnlisted:
		result.Cc = append(result.Cc, s.followersURLForLocalUser(note.AuthorID))
		deliverToFollowers = true
	case types.NoteVisibilityFollower:
		result.To = append(result.To, s.followersURLForLocalUser(note.AuthorID))
		deliverToFollowers = true
	}

	if needInboxes && deliverToFollowers {
		var follows []models.ActualUserFollow
		if err := s.DB(ctx).Where("followed_id = ?", note.AuthorID).Find(&follows).Error; err != nil {
			return calculateToAndCcResult{}, fmt.Errorf("failed to get followers for user %s: %w", note.AuthorID, err)
		}
		// TODO: inbox 取得するだけなので、Join で処理できるはず
		for _, follow := range follows {
			followerUser, err := s.FindApubUserByID(ctx, follow.FollowerID)
			if err != nil {
				return calculateToAndCcResult{}, fmt.Errorf("error fetching follower user %s: %w", follow.FollowerID, err)
			}
			if followerUser == nil {
				return calculateToAndCcResult{}, fmt.Errorf("follower user not found: %s", follow.FollowerID)
			}
			addInbox(followerUser.Apub.PreferredInbox())
		}
	}

	// Mentioned users
	for _, mention := range mentions {
		mentionUserID := mention.TargetUserID
		mentionedUser, err := s.FindApubUserByID(ctx, mentionUserID)
		if err != nil {
			return calculateToAndCcResult{}, fmt.Errorf("error fetching mentioned user %s: %w", mentionUserID, err)
		}
		if mentionedUser == nil {
			return calculateToAndCcResult{}, fmt.Errorf("mentioned user not found: %s", mentionUserID)
		}
		result.To = append(result.To, mentionedUser.Apub.URL)
		addInbox(mentionedUser.Apub.PreferredInbox())
	}

	return result, nil
}

func (s *State) publishNoteToApub(ctx context.Context, noteID types.NoteID) error {
	note, err := s.findApubNoteByIDWithDeleted(ctx, noteID)
	if err != nil {
		return fmt.Errorf("failed to fine note: %w", err)
	}
	if note == nil {
		return fmt.Errorf("delivery target note not found")
	}

	noteAuthor, err := s.FindApubUserByID(ctx, note.Data.Basic.Author.ID)
	if err != nil {
		return fmt.Errorf("failed to find note author: %w", err)
	}
	if noteAuthor == nil {
		return fmt.Errorf("note author not found for note: %s", note.Data.Basic.ID)
	}

	// check if deleted
	if note.Deleted {
		// TODO: deliver Tombstone
		return nil
	}

	noteObject := apub.NewNoteObject(&note.Data)

	// CREATE or UPDATE
	var activity any
	if noteObject.Updated == nil {
		create := apub.NewCreateActivity(noteObject.AsCreatableObject())
		activity = create
	} else {
		update := apub.NewUpdateActivity(noteObject.AsUpdatableObject())
		activity = update
	}

	withContext := apub.WithContext(activity)
	return s.delivery.QueueActivity(ctx, withContext, noteAuthor, note.Data.Apub.Inboxes)
}

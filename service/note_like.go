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

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm/clause"
)

var (
	ErrAlreadyReacted         = NewServiceError(400, "already reacted to note")
	ErrReactionTargetNotFound = NewServiceError(400, "reaction target not found")
	ErrReactionerNotFound     = NewServiceError(400, "reactioner not found")

	DefaultNoteReaction types.NoteReactionContent
)

func init() {
	d, err := types.NewEmojiNoteReaction("❤")
	if err != nil {
		panic(err)
	}
	DefaultNoteReaction = d
}

func (s *State) NoteReactionAdd(
	ctx context.Context,
	userID types.UserID,
	noteID types.NoteID,
	reaction types.NoteReactionContent,
) error {
	user, err := s.FindUserByID(ctx, userID)
	if err != nil {
		return err
	}
	if user == nil {
		return ErrReactionerNotFound
	}

	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, &userID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return ErrReactionTargetNotFound
	}

	result := s.DB(ctx).Clauses(
		clause.OnConflict{
			DoNothing: true,
		},
	).Create(&models.NoteReaction{
		NoteID:   noteID,
		UserID:   userID,
		Reaction: reaction.ReactionAsText(),
	})
	if result.Error != nil {
		return result.Error
	}
	if result.RowsAffected == 0 {
		return ErrAlreadyReacted
	}

	if user.IsLocal() && note.Author.IsRemote() {
		// TODO: apub Like
	}

	return nil
}

func (s *State) NoteReactionRemove(
	ctx context.Context,
	userID types.UserID,
	noteID types.NoteID,
) error {
	user, err := s.FindUserByID(ctx, userID)
	if err != nil {
		return err
	}
	if user == nil {
		return ErrReactionerNotFound
	}

	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, &userID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return ErrReactionTargetNotFound
	}

	result := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Delete(&models.NoteReaction{})
	if result.Error != nil {
		return result.Error
	}
	if result.RowsAffected == 0 {
		// No reaction to remove
		return nil
	}

	if user.IsLocal() && note.Author.IsRemote() {
		// TODO: apub Undo(Like)
	}

	return nil
}

func (s *State) NoteBookmarkAdd(
	ctx context.Context,
	userID types.UserID,
	noteID types.NoteID,
) error {
	user, err := s.FindUserByID(ctx, userID)
	if err != nil {
		return err
	}
	if user == nil {
		return ErrReactionerNotFound
	}

	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, &userID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return ErrReactionTargetNotFound
	}

	result := s.DB(ctx).Clauses(
		clause.OnConflict{
			DoNothing: true,
		},
	).Create(&models.NoteBookmark{
		NoteID: noteID,
		UserID: userID,
	})
	if result.Error != nil {
		return result.Error
	}

	// Bookmark is private, so no federation action is needed

	return nil
}

func (s *State) NoteBookmarkRemove(
	ctx context.Context,
	userID types.UserID,
	noteID types.NoteID,
) error {
	user, err := s.FindUserByID(ctx, userID)
	if err != nil {
		return err
	}
	if user == nil {
		return ErrReactionerNotFound
	}

	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, &userID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return ErrReactionTargetNotFound
	}

	result := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Delete(&models.NoteBookmark{})
	if result.Error != nil {
		return result.Error
	}

	// Bookmark is private, so no federation action is needed

	return nil
}

func (s *State) checkNoteReacted(ctx context.Context, noteID types.NoteID, userID types.UserID) (*string, error) {
	var reaction []models.NoteReaction
	if err := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Limit(1).Find(&reaction).Error; err != nil {
		return nil, err
	}
	if len(reaction) == 0 {
		return nil, nil
	}
	return &reaction[0].Reaction, nil
}

func (s *State) checkNoteBookmarked(ctx context.Context, noteID types.NoteID, userID types.UserID) (bool, error) {
	var count int64
	if err := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Model(&models.NoteBookmark{}).Count(&count).Error; err != nil {
		return false, err
	}
	return count > 0, nil
}

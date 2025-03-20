package service

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
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
	).Create(&db.NoteReaction{
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

	result := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Delete(&db.NoteReaction{})
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
	).Create(&db.NoteBookmark{
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

	result := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).Delete(&db.NoteBookmark{})
	if result.Error != nil {
		return result.Error
	}

	// Bookmark is private, so no federation action is needed

	return nil
}

func (s *State) checkNoteReacted(ctx context.Context, noteID types.NoteID, userID types.UserID) (*string, error) {
	var reaction db.NoteReaction
	if err := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).First(&reaction).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, nil
		}
		return nil, err
	}
	return &reaction.Reaction, nil
}

func (s *State) checkNoteBookmarked(ctx context.Context, noteID types.NoteID, userID types.UserID) (bool, error) {
	var bookmark db.NoteBookmark
	if err := s.DB(ctx).Where("note_id = ? AND user_id = ?", noteID, userID).First(&bookmark).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

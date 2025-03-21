package service

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"
)

var (
	ErrNoteDeletePermission = NewServiceError(http.StatusBadRequest, "note not found or not note author")
)

func (s *State) DeleteNoteByID(ctx context.Context, userID types.UserID, noteID types.NoteID) error {
	err := s.WithTransaction(func(tx *State) error {
		var note db.Note
		if err := tx.DB(ctx).Where("id = ? AND author_id = ? AND deleted_at IS NULL", noteID, userID).Clauses(
			clause.Locking{Strength: "UPDATE"},
		).First(&note).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return ErrNoteDeletePermission
			}
			return err
		}

		if err := tx.DB(ctx).Where("id = ?", note.ID).Model(&db.Note{}).Update("deleted_at", time.Now()).Error; err != nil {
			return err
		}

		return nil
	})
	if err != nil {
		return err
	}

	// TODO: apub Delete

	return nil
}

func (s *State) DeleteRenote(ctx context.Context, userID types.UserID, renoteTargetID types.NoteID) error {
	err := s.WithTransaction(func(tx *State) error {
		var renote db.Note
		if err := tx.DB(ctx).Where("renote_of_id = ? AND author_id = ? AND content IS NULL", renoteTargetID, userID).Clauses(
			clause.Locking{Strength: "UPDATE"},
		).First(&renote).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return ErrNoteDeletePermission
			}
			return err
		}

		if err := tx.DB(ctx).Delete(&renote).Error; err != nil {
			return err
		}

		return nil
	})
	if err != nil {
		return err
	}

	// TODO: apub Undo(Announce)

	return nil
}

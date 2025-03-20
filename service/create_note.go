package service

import (
	"database/sql"
	"errors"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

var (
	InsertNewNote *UpsertTarget = nil

	ErrAuthorNotFound           = NewServiceError(400, "author not found")
	ErrUpdateTargetNoteNotFound = NewServiceError(400, "update target note not found")
	ErrVisibilityRequired       = NewServiceError(400, "visibility required")
)

type UpsertTarget struct {
	noteID  *types.NoteID
	noteURL *string
}

type CreateNoteParams struct {
	Content    types.NoteContent
	Visibility *types.NoteVisibility // insert time only
	ReplyToID  *types.NoteID
	RenoteOfID *types.NoteID
	CreatedAt  *time.Time // nil for now
	Uploads    []types.UploadID
	Hashtags   []string       // nil = infer from content
	Mentions   []types.UserID // nil = infer from content
	ViewURL    *string
	Sensitive  bool
}

func (s *ServiceState) UpsertNote(
	upsertTarget *UpsertTarget,
	author types.UserID,
	params CreateNoteParams,
) (types.NoteID, error) {
	var upsertedNoteID types.NoteID
	err := s.WithTransaction(func(tx *ServiceState) error {
		authorUser, err := tx.FindUserByID(author)
		if err != nil {
			return err
		}
		if authorUser == nil {
			return ErrAuthorNotFound
		}

		updatedNote, err := tx.findNoteForUpsert(upsertTarget)
		if err != nil {
			return err
		}
		isUpdate := updatedNote != nil

		if updatedNote == nil && params.Visibility == nil {
			return ErrVisibilityRequired
		}

		if updatedNote == nil {
			// new note (local or remote)
			updatedNote = &db.Note{
				ID: types.NewNoteID(),
			}
			if upsertTarget != nil && upsertTarget.noteURL != nil {
				// remote note; set URL
				updatedNote.URL = sql.NullString{String: *upsertTarget.noteURL, Valid: true}
			}
		}
		upsertedNoteID = updatedNote.ID

		hashtags, err := tx.findHashtagsInNoteContent(params.Content, params.Hashtags)
		if err != nil {
			return err
		}
		mentions, err := tx.findMentionsInNoteContent(params.Content, params.Mentions)
		if err != nil {
			return err
		}

		var remoteURL *string
		if upsertTarget != nil && upsertTarget.noteURL != nil {
			remoteURL = upsertTarget.noteURL
		}

		// Upsert note
		setNoteModelForUpsert(updatedNote, author, remoteURL, params, isUpdate)
		if err := tx.DB().Save(updatedNote).Error; err != nil {
			return err
		}

		// Update hashtags
		if err := tx.DB().Where("note_id = ?", updatedNote.ID).Delete(&db.NoteTag{}).Error; err != nil {
			return err
		}
		for _, hashtag := range hashtags {
			tagID, err := tx.getOrCreateTagID(hashtag)
			if err != nil {
				return err
			}
			if err := tx.DB().Create(&db.NoteTag{
				NoteID: updatedNote.ID,
				TagID:  tagID,
			}).Error; err != nil {
				return err
			}
		}

		// Update mentions
		if err := tx.DB().Where("note_id = ?", updatedNote.ID).Delete(&db.NoteMention{}).Error; err != nil {
			return err
		}
		for _, mention := range mentions {
			if err := tx.DB().Create(&db.NoteMention{
				NoteID:       updatedNote.ID,
				TargetUserID: mention,
			}).Error; err != nil {
				return err
			}
		}

		// Insert uploads
		if !isUpdate {
			for _, uploadID := range params.Uploads {
				if err := tx.DB().Create(&db.NoteUpload{
					NoteID:   updatedNote.ID,
					UploadID: uploadID,
				}).Error; err != nil {
					return err
				}
			}
		}

		return nil
	})

	if err != nil {
		return types.NoteID{}, err
	}

	// TODO: reply and mention notifications

	// TODO: apub federation

	return upsertedNoteID, nil
}

func setNoteModelForUpsert(model *db.Note, author types.UserID, url *string, params CreateNoteParams, isUpdate bool) {
	model.URL = stringPtrToSql(url)
	model.ViewURL = stringPtrToSql(params.ViewURL)
	model.AuthorID = author
	model.Content = stringToSql(params.Content.Data)
	model.ContentType = stringToSql(string(params.Content.Type))

	// Timestamps
	now := time.Now()
	if isUpdate {
		if params.CreatedAt != nil {
			model.UpdatedAt = timePtrToSql(params.CreatedAt)
		} else {
			model.UpdatedAt = timePtrToSql(&now)
		}
	} else {
		if params.CreatedAt != nil {
			model.CreatedAt = *params.CreatedAt
		} else {
			model.CreatedAt = now
		}
	}

	model.Visibility = string(*params.Visibility)
	model.ReplyToID = params.ReplyToID
	model.Sensitive = params.Sensitive
	if model.URL.Valid {
		// is remote
		model.FetchedAt = timePtrToSql(&now)
	}
}

func (s *ServiceState) getOrCreateTagID(name string) (int, error) {
	var tag db.Tag
	if err := s.DB().Where("name = ?", name).First(&tag).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// create new tag
			tag = db.Tag{
				Name: name,
			}
			if err := s.DB().Create(&tag).Error; err != nil {
				return 0, err
			}
		} else {
			return 0, err
		}
	}

	return tag.ID, nil
}

func (s *ServiceState) findHashtagsInNoteContent(content types.NoteContent, override []string) ([]string, error) {
	if override != nil {
		return override, nil
	}

	switch content.Type {
	case types.NoteContentTypePlain:
		return nil, nil //TODO
	default:
		return nil, nil // TODO
	}
}

func (s *ServiceState) findMentionsInNoteContent(content types.NoteContent, override []types.UserID) ([]types.UserID, error) {
	if override != nil {
		return override, nil
	}

	switch content.Type {
	case types.NoteContentTypePlain:
		return nil, nil // TODO
	default:
		return nil, nil // TODO
	}
}

func (s *ServiceState) findNoteForUpsert(upsertTarget *UpsertTarget) (*db.Note, error) {
	if upsertTarget == nil {
		// new local note
		return nil, nil
	}

	// conditions
	// 1. match ID or URL
	// 2. not deleted
	// 3. not a renote

	if upsertTarget.noteID != nil {
		var note db.Note
		if err := s.DB().Where("id = ? AND deleted_at IS NULL AND renote_of_id IS NULL", upsertTarget.noteID).First(&note).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return nil, ErrUpdateTargetNoteNotFound
			}
			return nil, err
		}
		// existing local note
		return &note, nil
	}

	if upsertTarget.noteURL != nil {
		var note db.Note
		if err := s.DB().Where("url = ? AND deleted_at IS NULL AND renote_of_id IS NULL", upsertTarget.noteURL).First(&note).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				// new remote note
				return nil, nil
			}
			// existing remote note
			return nil, err
		}
		return &note, nil
	}

	panic("unreachable")
}

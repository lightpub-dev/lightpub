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
	"database/sql"
	"errors"
	"log/slog"
	"time"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/service/notification"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

var (
	InsertNewNote *UpsertTarget = nil

	ErrAuthorNotFound           = NewServiceError(400, "author not found")
	ErrUpdateTargetNoteNotFound = NewServiceError(400, "update target note not found")
	ErrVisibilityRequired       = NewServiceError(400, "visibility required")
	ErrRenoteVisibility         = NewServiceError(400, "renote (and renote target) visibility should be public or unlisted")
	ErrAlreadyRenoted           = NewServiceError(400, "already renoted")
	ErrRenoteTargetNotFound     = NewServiceError(400, "renote target not found")
	ErrReplyTargetNotFound      = NewServiceError(400, "reply target not found")
	ErrEditNotePermission       = NewServiceError(403, "edit note permission denied")
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

func (s *State) CreateNote(
	ctx context.Context,
	author types.UserID,
	params CreateNoteParams,
) (types.NoteID, error) {
	return s.UpsertNote(ctx, nil, author, params)
}

func (s *State) UpsertNote(
	ctx context.Context,
	upsertTarget *UpsertTarget,
	author types.UserID,
	params CreateNoteParams,
) (types.NoteID, error) {
	var (
		upsertedNoteID   types.NoteID
		repliedNote      *types.SimpleNote
		mentionedUserIDs []types.UserID
	)
	err := s.WithTransaction(func(tx *State) error {
		authorUser, err := tx.FindUserByID(ctx, author)
		if err != nil {
			return err
		}
		if authorUser == nil {
			return ErrAuthorNotFound
		}

		updatedNote, err := tx.findNoteForUpsert(ctx, upsertTarget)
		if err != nil {
			return err
		}
		isUpdate := updatedNote != nil

		// update author check
		if updatedNote != nil && updatedNote.AuthorID != author {
			return ErrEditNotePermission
		}

		if updatedNote == nil && params.Visibility == nil {
			return ErrVisibilityRequired
		}

		if updatedNote == nil {
			// new note (local or remote)
			updatedNote = &models.Note{
				ID: types.NewNoteID(),
			}
			if upsertTarget != nil && upsertTarget.noteURL != nil {
				// remote note; set URL
				updatedNote.URL = sql.NullString{String: *upsertTarget.noteURL, Valid: true}
			}
		}
		upsertedNoteID = updatedNote.ID

		hashtags, err := tx.findHashtagsInNoteContent(ctx, params.Content, params.Hashtags)
		if err != nil {
			return err
		}
		mentions, err := tx.findMentionsInNoteContent(ctx, params.Content, params.Mentions)
		if err != nil {
			return err
		}
		mentionedUserIDs = mentions

		// reply note visibility check
		if params.ReplyToID != nil {
			replyNote, err := tx.FindNoteByIDWithVisibilityCheck(ctx, &author, *params.ReplyToID)
			if err != nil {
				return err
			}
			if replyNote == nil {
				return ErrReplyTargetNotFound
			}
			repliedNote = replyNote
		}

		var remoteURL *string
		if upsertTarget != nil && upsertTarget.noteURL != nil {
			remoteURL = upsertTarget.noteURL
		}

		// Upsert note
		setNoteModelForUpsert(updatedNote, author, remoteURL, params, isUpdate)
		if err := tx.DB(ctx).Save(updatedNote).Error; err != nil {
			return err
		}

		// Update hashtags
		if err := tx.DB(ctx).Where("note_id = ?", updatedNote.ID).Delete(&models.NoteTag{}).Error; err != nil {
			return err
		}
		for _, hashtag := range hashtags {
			tagID, err := tx.getOrCreateTagID(ctx, hashtag)
			if err != nil {
				return err
			}
			if err := tx.DB(ctx).Create(&models.NoteTag{
				NoteID: updatedNote.ID,
				TagID:  tagID,
			}).Error; err != nil {
				return err
			}
		}

		// Update mentions
		if err := tx.DB(ctx).Where("note_id = ?", updatedNote.ID).Delete(&models.NoteMention{}).Error; err != nil {
			return err
		}
		for _, mention := range mentions {
			if err := tx.DB(ctx).Create(&models.NoteMention{
				NoteID:       updatedNote.ID,
				TargetUserID: mention,
			}).Error; err != nil {
				return err
			}
		}

		// Insert uploads
		if !isUpdate {
			for _, uploadID := range params.Uploads {
				if err := tx.DB(ctx).Create(&models.NoteUpload{
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

	// send notifications
	notifiedUserIDs := make(map[types.UserID]struct{})
	isAlreadyNotified := func(userID types.UserID) bool {
		_, ok := notifiedUserIDs[userID]
		return ok
	}
	// reply notification
	if repliedNote != nil && repliedNote.Author.IsLocal() {
		notification := &notification.Replied{
			ReplierUserID: author,
			ReplyNoteID:   upsertedNoteID,
			RepliedNoteID: repliedNote.ID,
		}
		if err := s.AddNotification(ctx, repliedNote.Author.ID, notification); err != nil {
			slog.ErrorContext(ctx, "failed to send reply notification", "noteID", upsertedNoteID, "err", err)
		}
		notifiedUserIDs[repliedNote.Author.ID] = struct{}{}
	}
	// mention notification
	for _, mentionUserID := range mentionedUserIDs {
		mentionUser, err := s.FindUserByID(ctx, mentionUserID)
		if err != nil {
			return types.NoteID{}, err
		}
		if mentionUser == nil {
			slog.ErrorContext(ctx, "failed to send mention notification: user not found", "noteID", upsertedNoteID, "userID", mentionUserID)
			continue
		}
		if mentionUser.ID == author || mentionUser.IsRemote() || isAlreadyNotified(mentionUser.ID) {
			continue
		}
		notification := &notification.Mentioned{
			MentionerUserID: author,
			MentionNoteID:   upsertedNoteID,
		}
		if err := s.AddNotification(ctx, mentionUser.ID, notification); err != nil {
			slog.ErrorContext(ctx, "failed to send mention notification", "noteID", upsertedNoteID, "userID", mentionUserID, "err", err)
		}
	}

	if err := s.publishNoteToApub(ctx, upsertedNoteID); err != nil {
		slog.WarnContext(ctx, "failed to publish note to apub", "noteID", upsertedNoteID, "err", err)
	}

	return upsertedNoteID, nil
}

func setNoteModelForUpsert(model *models.Note, author types.UserID, url *string, params CreateNoteParams, isUpdate bool) {
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

	if !isUpdate {
		// Non-updatable fields
		model.Visibility = *params.Visibility
		model.ReplyToID = params.ReplyToID
		model.Sensitive = params.Sensitive
	}

	if model.URL.Valid {
		// is remote
		model.FetchedAt = timePtrToSql(&now)
	}
}

func (s *State) getOrCreateTagID(ctx context.Context, name string) (int, error) {
	var tag models.Tag
	if err := s.DB(ctx).Where("name = ?", name).First(&tag).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// create new tag
			tag = models.Tag{
				Name: name,
			}
			if err := s.DB(ctx).Create(&tag).Error; err != nil {
				return 0, err
			}
		} else {
			return 0, err
		}
	}

	return tag.ID, nil
}

func (s *State) findHashtagsInNoteContent(ctx context.Context, content types.NoteContent, override []string) ([]string, error) {
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

func (s *State) findMentionsInNoteContent(ctx context.Context, content types.NoteContent, override []types.UserID) ([]types.UserID, error) {
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

func (s *State) findNoteForUpsert(ctx context.Context, upsertTarget *UpsertTarget) (*models.Note, error) {
	if upsertTarget == nil {
		// new local note
		return nil, nil
	}

	// conditions
	// 1. match ID or URL
	// 2. not deleted
	// 3. not a renote

	if upsertTarget.noteID != nil {
		var note models.Note
		if err := s.DB(ctx).WithContext(ctx).Where("id = ? AND deleted_at IS NULL AND renote_of_id IS NULL", upsertTarget.noteID).First(&note).Error; err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				return nil, ErrUpdateTargetNoteNotFound
			}
			return nil, err
		}
		// existing local note
		return &note, nil
	}

	if upsertTarget.noteURL != nil {
		var note models.Note
		if err := s.DB(ctx).WithContext(ctx).Where("url = ? AND deleted_at IS NULL AND renote_of_id IS NULL", upsertTarget.noteURL).First(&note).Error; err != nil {
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

func (s *State) CreateRenote(ctx context.Context, authorID types.UserID, targetNoteID types.NoteID, visibility types.NoteVisibility) (types.NoteID, error) {
	// renote visibility check
	if !visibility.ValidAsRenote() {
		return types.NoteID{}, ErrRenoteVisibility
	}

	// author check
	author, err := s.FindUserByID(ctx, authorID)
	if err != nil {
		return types.NoteID{}, err
	}
	if author == nil {
		return types.NoteID{}, ErrAuthorNotFound
	}

	// check if author has already renoted the target note
	var existingRenote models.Note
	err = s.DB(ctx).Where("author_id = ? AND renote_of_id = ?", authorID, targetNoteID).First(&existingRenote).Error
	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return types.NoteID{}, err
	}
	if err == nil {
		return types.NoteID{}, ErrAlreadyRenoted
	}

	// target note check
	targetNote, err := s.FindNoteByIDWithVisibilityCheck(ctx, &authorID, targetNoteID)
	if err != nil {
		return types.NoteID{}, err
	}
	if targetNote == nil {
		return types.NoteID{}, ErrUpdateTargetNoteNotFound
	}

	// target note visibility check
	if !targetNote.Visibility.AcceptRenote() {
		return types.NoteID{}, ErrRenoteVisibility
	}

	renoteID := types.NewNoteID()
	now := time.Now()
	err = s.DB(ctx).Create(&models.Note{
		ID:         renoteID,
		AuthorID:   authorID,
		CreatedAt:  now,
		Visibility: visibility,
		RenoteOfID: &targetNoteID,
	}).Error
	if err != nil {
		return types.NoteID{}, err
	}

	// notification (if renoted user is local)
	if targetNote.Author.IsLocal() && targetNote.Author.ID != authorID {
		notification := &notification.Renote{
			RenoterUserID: authorID,
			RenoteNoteID:  renoteID,
		}
		if err := s.AddNotification(ctx, targetNote.Author.ID, notification); err != nil {
			slog.ErrorContext(ctx, "failed to send renote notification", "noteID", renoteID, "userID", targetNote.Author.ID, "err", err)
		}
	}

	// federation (if renoted user is remote)
	if author.IsLocal() && targetNote.Author.IsRemote() {
		// TODO: apub Announce
	}

	return renoteID, nil
}

type NoteEditParams struct {
	Content types.NoteContent
}

func (s *State) EditNote(ctx context.Context, authorID types.UserID, noteID types.NoteID, newNote NoteEditParams) error {
	// author check
	author, err := s.FindUserByID(ctx, authorID)
	if err != nil {
		return err
	}
	if author == nil {
		return ErrAuthorNotFound
	}

	// note check
	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, &authorID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return ErrUpdateTargetNoteNotFound
	}
	if note.Author.ID != authorID {
		return ErrEditNotePermission
	}

	_, err = s.UpsertNote(ctx, &UpsertTarget{noteID: &noteID}, authorID, CreateNoteParams{
		Content: newNote.Content,
	})
	if err != nil {
		return err
	}

	return nil
}

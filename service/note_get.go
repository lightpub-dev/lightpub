package service

import (
	"context"
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

type noteAuthorPair struct {
	Note   db.Note
	Author db.User
}

func (s *State) findNoteByIDRaw(ctx context.Context, noteID types.NoteID, includeDeleted bool) (*noteAuthorPair, error) {
	var note db.Note
	if err := s.DB(ctx).Where("id = ?", noteID).Joins("Author").First(&note).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	if !includeDeleted && note.DeletedAt.Valid {
		return nil, nil
	}
	return &noteAuthorPair{
		Note:   note,
		Author: note.Author,
	}, nil
}

func (s *State) FindNoteByID(ctx context.Context, noteID types.NoteID) (*types.SimpleNote, error) {
	pair, err := s.findNoteByIDRaw(ctx, noteID, false)
	if err != nil {
		return nil, err
	}
	if pair == nil {
		return nil, nil
	}

	var uploads []db.NoteUpload
	if err := s.DB(ctx).Where("note_id = ?", noteID).Find(&uploads).Error; err != nil {
		return nil, err
	}
	uploadIDs := make([]types.UploadID, len(uploads))
	for i, upload := range uploads {
		uploadIDs[i] = upload.UploadID
	}

	note := &pair.Note
	author := &pair.Author

	var content *types.NoteContent
	if note.Content.Valid && note.ContentType.Valid {
		content = &types.NoteContent{
			Data: note.Content.String,
			Type: types.NoteContentType(note.ContentType.String),
		}
	}

	return &types.SimpleNote{
		ID: noteID,
		Author: types.NoteAuthor{
			ID:       author.ID,
			Username: author.Username,
			Domain:   author.Domain,
			Nickname: author.Nickname,
		},
		Content:    content,
		Visibility: types.NoteVisibility(note.Visibility),
		CreatedAt:  note.CreatedAt,
		UpdatedAt:  sqlToTimePtr(note.UpdatedAt),

		ReplyToID:  note.ReplyToID,
		RenoteOfID: note.RenoteOfID,
		DeletedAt:  sqlToTimePtr(note.DeletedAt),

		Sensitive: note.Sensitive,
		Uploads:   uploadIDs,
	}, nil
}

func (s *State) CheckNoteVisibility(ctx context.Context, viewerID *types.UserID, note types.SimpleNote) (bool, error) {
	switch note.Visibility {
	case types.NoteVisibilityPublic:
	case types.NoteVisibilityUnlisted:
		return true, nil
	case types.NoteVisibilityFollower:
		if viewerID == nil {
			return false, nil
		}
		followState, err := s.GetFollowState(ctx, *viewerID, note.Author.ID)
		if err != nil {
			return false, err
		}
		return followState == FollowStateYes, nil
	case types.NoteVisibilityPrivate:
		if viewerID == nil {
			return false, nil
		}
		isMentioned, err := s.checkNoteMentioned(ctx, note.ID, *viewerID)
		if err != nil {
			return false, err
		}
		return isMentioned, nil
	}

	panic("unreachable")
}

func (s *State) FindNoteByIDWithVisibilityCheck(ctx context.Context, viewerID *types.UserID, noteID types.NoteID) (*types.SimpleNote, error) {
	note, err := s.FindNoteByID(ctx, noteID)
	if err != nil {
		return nil, err
	}
	if note == nil {
		return nil, nil
	}

	visible, err := s.CheckNoteVisibility(ctx, viewerID, *note)
	if err != nil {
		return nil, err
	}
	if !visible {
		return nil, nil
	}
	return note, nil
}

func (s *State) checkNoteMentioned(ctx context.Context, noteID types.NoteID,
	targetUserID types.UserID) (bool, error) {
	var mentions db.NoteMention
	if err := s.DB(ctx).Where("note_id = ? AND target_user_id = ?", noteID, targetUserID).First(&mentions).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

package service

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) getNotesFromIDs(ctx context.Context, viewerID *types.UserID, noteIDs []types.NoteID) ([]types.DetailedNote, error) {
	notes := make([]types.DetailedNote, 0, len(noteIDs))
	// TODO: avoid N+1 queries
	for _, noteID := range noteIDs {
		note, err := s.FindNoteByIDWithDetails(ctx, viewerID, noteID)
		if err != nil {
			return nil, err
		}
		if note != nil {
			notes = append(notes, *note)
		}
	}
	return notes, nil
}

func (s *State) GetPublicTimeline(ctx context.Context, userID *types.UserID, limit uint64, beforeDate *time.Time) ([]types.DetailedNote, error) {
	noteIDs, err := s.getTimelineNoteIDs(ctx, userID, true, limit, beforeDate)
	if err != nil {
		return nil, err
	}
	return s.getNotesFromIDs(ctx, userID, noteIDs)
}

func (s *State) GetTimeline(ctx context.Context, userID types.UserID, limit uint64, beforeDate *time.Time) ([]types.DetailedNote, error) {
	noteIDs, err := s.getTimelineNoteIDs(ctx, &userID, false, limit, beforeDate)
	if err != nil {
		return nil, err
	}
	return s.getNotesFromIDs(ctx, &userID, noteIDs)
}

func (s *State) GetUserNotes(ctx context.Context, viewerID *types.UserID, userID types.UserID, limit uint64, beforeDate *time.Time) ([]types.DetailedNote, error) {
	var ids []struct {
		ID types.NoteID `db:"id"`
	}

	query := `CALL get_user_note_ids(?,?,?,?)`
	err := s.DB(ctx).Raw(query, viewerID, userID, limit, beforeDate).Scan(&ids).Error
	if err != nil {
		return nil, err
	}

	noteIDs := make([]types.NoteID, len(ids))
	for i, id := range ids {
		noteIDs[i] = id.ID
	}

	return s.getNotesFromIDs(ctx, viewerID, noteIDs)
}

func (s *State) GetNoteReplies(ctx context.Context, viewerID *types.UserID, targetNoteID types.NoteID, limit uint64, beforeDate *time.Time) ([]types.DetailedNote, error) {
	noteIDs, err := s.getNoteReplyIDs(ctx, viewerID, targetNoteID, limit, beforeDate)
	if err != nil {
		return nil, err
	}
	return s.getNotesFromIDs(ctx, viewerID, noteIDs)
}

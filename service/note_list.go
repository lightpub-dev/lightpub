package service

import (
	"context"
	"time"

	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) getNoteIDsGeneralized(ctx context.Context, viewerID *types.UserID, includeSelf bool, includePublic bool, includeUnlisted bool, limitReplyToID *types.NoteID, limit uint64, beforeDate *time.Time) ([]types.NoteID, error) {
	var ids []struct {
		ID types.NoteID `db:"id"`
	}

	query := `CALL get_note_ids_generalized(?,?,?,?,?,?,?)`
	err := s.DB(ctx).Raw(query, viewerID, includeSelf, includePublic, includeUnlisted, limitReplyToID, limit, beforeDate).Scan(&ids).Error
	if err != nil {
		return nil, err
	}

	noteIDs := make([]types.NoteID, len(ids))
	for i, id := range ids {
		noteIDs[i] = id.ID
	}
	return noteIDs, nil
}

func (s *State) getTimelineNoteIDs(ctx context.Context, viewerID *types.UserID, includePublic bool, limit uint64, beforeDate *time.Time) ([]types.NoteID, error) {
	return s.getNoteIDsGeneralized(ctx, viewerID, true, includePublic, false, nil, limit, beforeDate)
}

func (s *State) getNoteReplyIDs(ctx context.Context, viewerID *types.UserID, targetNoteID types.NoteID, limit uint64, beforeDate *time.Time) ([]types.NoteID, error) {
	return s.getNoteIDsGeneralized(ctx, viewerID, true, true, true, &targetNoteID, limit, beforeDate)
}

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

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
)

type TrendEntry struct {
	Hashtag string
	Count   uint64
}

func (s *State) GetTrendingTags(ctx context.Context, limit uint64) ([]TrendEntry, error) {
	var tags []models.TrendingTag
	if err := s.DB(ctx).Model(&models.TrendingTag{}).Limit(int(limit)).Find(&tags).Error; err != nil {
		return nil, err
	}

	entries := make([]TrendEntry, len(tags))
	for i, tag := range tags {
		entries[i] = TrendEntry{
			Hashtag: tag.Name,
			Count:   uint64(tag.NoteCount),
		}
	}

	return entries, nil
}

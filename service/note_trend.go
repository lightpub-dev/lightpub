package service

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
)

type TrendEntry struct {
	Hashtag string
	Count   uint64
}

func (s *State) GetTrendingTags(ctx context.Context, limit uint64) ([]TrendEntry, error) {
	var tags []db.TrendingTag
	if err := s.DB(ctx).Model(&db.TrendingTag{}).Limit(int(limit)).Find(&tags).Error; err != nil {
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

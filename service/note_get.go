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
		return followState == types.FollowStateYes, nil
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

func (s *State) FindNoteByIDWithDetails(ctx context.Context, viewerID *types.UserID, noteID types.NoteID) (*types.DetailedNote, error) {
	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, viewerID, noteID)
	if err != nil {
		return nil, err
	}
	if note == nil {
		return nil, nil
	}

	details, err := s.getNoteDetails(ctx, viewerID, *note)
	if err != nil {
		return nil, err
	}

	return &types.DetailedNote{
		Basic:   *note,
		Details: details,
	}, nil
}

func (s *State) getNoteDetails(ctx context.Context, viewerID *types.UserID, note types.SimpleNote) (types.NoteDetails, error) {
	details := types.NoteDetails{}

	if viewerID != nil {
		renoted, err := s.checkNoteRenoted(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}
		details.Renoted = &renoted

		reaction, err := s.checkNoteReacted(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}
		details.Reacted = reaction

		bookmarked, err := s.checkNoteBookmarked(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}
		details.Bookmarked = &bookmarked
	}

	var (
		replyCount       int64
		renoteCount      int64
		reactionCountRaw []struct {
			Reaction string `db:"reaction"`
			Count    int64  `db:"count"`
		}
	)
	if err := s.DB(ctx).Model(&db.Note{}).Where("reply_to_id = ? AND deleted_at IS NULL", note.ID).Count(&replyCount).Error; err != nil {
		return details, err
	}
	if err := s.DB(ctx).Model(&db.Note{}).Where("renote_of_id = ? AND content IS NULL AND deleted_at IS NULL", note.ID).Count(&renoteCount).Error; err != nil {
		return details, err
	}
	if err := s.DB(ctx).Model(&db.NoteReaction{}).Select("reaction, COUNT(*) AS count").Where("note_id = ?", note.ID).Group("reaction").Scan(&reactionCountRaw).Error; err != nil {
		return details, err
	}
	reactionCount := make(map[string]uint64, len(reactionCountRaw))
	for _, raw := range reactionCountRaw {
		reactionCount[raw.Reaction] = uint64(raw.Count)
	}
	details.ReplyCount = uint64(replyCount)
	details.RenoteCount = uint64(renoteCount)
	details.ReactionCount = reactionCount

	var err error
	details.Hashtags, err = s.getNoteHashtags(ctx, note.ID)
	if err != nil {
		return details, err
	}
	details.Mentions, err = s.getNoteMentions(ctx, note.ID)
	if err != nil {
		return details, err
	}

	return details, nil
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

func (s *State) checkNoteRenoted(ctx context.Context, noteID types.NoteID, renoterID types.UserID) (bool, error) {
	var renote db.Note
	if err := s.DB(ctx).Where("renote_of_id = ? AND author_id = ? AND content IS NULL", noteID, renoterID).First(&renote).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

func (s *State) getNoteHashtags(ctx context.Context, noteID types.NoteID) ([]string, error) {
	var hashtags []db.NoteTag
	if err := s.DB(ctx).Where("note_id = ?", noteID).Joins("Tag").Find(&hashtags).Error; err != nil {
		return nil, err
	}
	tagNames := make([]string, len(hashtags))
	for i, hashtag := range hashtags {
		tagNames[i] = hashtag.Tag.Name
	}
	return tagNames, nil
}

func (s *State) getNoteMentions(ctx context.Context, noteID types.NoteID) ([]types.NoteMention, error) {
	var mentions []db.NoteMention
	if err := s.DB(ctx).Where("note_id = ?", noteID).Joins("TargetUser").Find(&mentions).Error; err != nil {
		return nil, err
	}
	mentionPairs := make([]types.NoteMention, len(mentions))
	for i, mention := range mentions {
		mentionPairs[i] = types.NoteMention{
			ID:       mention.TargetUserID,
			Username: mention.TargetUser.Username,
			Nickname: mention.TargetUser.Nickname,
			Domain:   mention.TargetUser.Domain,
		}
	}
	return mentionPairs, nil
}

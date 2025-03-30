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
	"errors"
	"fmt"
	"log/slog"
	"net/http"

	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/types"
	"github.com/microcosm-cc/bluemonday"
	"gorm.io/gorm"
)

var (
	bioSanitizer  = bluemonday.UGCPolicy()
	noteSanitizer = bluemonday.UGCPolicy()

	ErrNoteNotFound = NewServiceError(http.StatusNotFound, "note not found")
)

type noteAuthorPair struct {
	Note   models.Note
	Author models.User
}

func (s *State) findNoteByIDRaw(ctx context.Context, noteID types.NoteID, includeDeleted bool) (*noteAuthorPair, error) {
	var note models.Note
	if err := s.DB(ctx).Unscoped().Where("notes.id = ?", noteID).Joins("Author").First(&note).Error; err != nil {
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

func (s *State) simpleNoteFromRawNote(ctx context.Context, pair noteAuthorPair) (*types.SimpleNote, error) {
	noteID := pair.Note.ID

	var uploads []models.NoteUpload
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
		contentType := types.NoteContentType(note.ContentType.String)
		cleanContent, err := renderNoteContent(note.Content.String, contentType)
		if err != nil {
			return nil, err
		}
		content = &types.NoteContent{
			Data: cleanContent,
			Type: contentType,

			Source: note.Content.String,
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

func (s *State) FindNoteByID(ctx context.Context, noteID types.NoteID) (*types.SimpleNote, error) {
	pair, err := s.findNoteByIDRaw(ctx, noteID, false)
	if err != nil {
		return nil, err
	}
	if pair == nil {
		return nil, nil
	}

	return s.simpleNoteFromRawNote(ctx, *pair)
}

func (s *State) findApubURLByNoteID(ctx context.Context, noteID types.NoteID, includeDeleted bool) (string, error) {
	pair, err := s.findNoteByIDRaw(ctx, noteID, includeDeleted)
	if err != nil {
		return "", err
	}
	if pair == nil {
		return "", ErrNoteNotFound
	}

	if pair.Note.URL.Valid {
		return pair.Note.URL.String, nil
	}
	return s.urlForLocalNote(pair.Note.ID), nil
}

type MaybeDeletedApubNote struct {
	Deleted bool
	Data    types.ApubNote
}

func (s *State) findApubNoteByIDWithDeleted(ctx context.Context, noteID types.NoteID) (*MaybeDeletedApubNote, error) {
	pair, err := s.findNoteByIDRaw(ctx, noteID, true)
	if err != nil {
		return nil, err
	}
	if pair == nil {
		return nil, nil
	}

	apubNote, err := s.apubNoteFromRawNote(ctx, *pair)
	if err != nil {
		return nil, err
	}

	return &MaybeDeletedApubNote{
		Deleted: pair.Note.DeletedAt.Valid,
		Data:    apubNote,
	}, nil
}

func (s *State) findApubNoteByID(ctx context.Context, noteID types.NoteID) (*types.ApubNote, error) {
	pair, err := s.findNoteByIDRaw(ctx, noteID, false)
	if err != nil {
		return nil, err
	}
	if pair == nil {
		return nil, nil
	}

	n, err := s.apubNoteFromRawNote(ctx, *pair)
	if err != nil {
		return nil, err
	}
	return &n, nil
}

func (s *State) apubNoteFromRawNote(ctx context.Context, pair noteAuthorPair) (types.ApubNote, error) {
	simpleNote, err := s.simpleNoteFromRawNote(ctx, pair)
	if err != nil {
		return types.ApubNote{}, err
	}

	var (
		url       string
		viewURL   string
		authorURL string
	)
	if simpleNote.Author.IsLocal() {
		url = s.urlForLocalNote(simpleNote.ID)
		viewURL = s.viewURLForLocalNote(simpleNote.ID)
		authorURL = s.urlForLocalUser(simpleNote.Author.ID)
	} else {
		return types.ApubNote{}, fmt.Errorf("remote notes are not supported")
	}

	// hashtags
	hashtags, err := s.getNoteHashtags(ctx, simpleNote.ID)
	if err != nil {
		return types.ApubNote{}, err
	}
	apubHashtags := make([]types.ApubHashtag, 0, len(hashtags))
	for _, hashtag := range hashtags {
		apubHashtags = append(apubHashtags, types.ApubHashtag{
			Name:        hashtag,
			TimelineURL: s.viewURLForHashtag(hashtag),
		})
	}

	// mentions
	mentions, err := s.getNoteMentions(ctx, simpleNote.ID)
	if err != nil {
		return types.ApubNote{}, err
	}
	apubMentions := make([]types.ApubMention, 0, len(mentions))
	for _, mention := range mentions {
		userURL, err := s.findApubURLForUserID(ctx, mention.ID)
		if err != nil {
			slog.WarnContext(ctx, "error fetching mentioned user (skipped)", "noteID", simpleNote.ID, "mentionID", mention.ID, "err", err)
			continue
		}
		apubMentions = append(apubMentions, types.ApubMention{
			Specifier: mention.Specifier(),
			URL:       userURL,
		})
	}

	// apubUploads
	apubUploads := make([]types.ApubUpload, 0, len(simpleNote.Uploads))
	for _, uploadID := range simpleNote.Uploads {
		upload, err := s.GetUpload(ctx, uploadID)
		if err != nil {
			return types.ApubNote{}, fmt.Errorf("error fetching upload %s: %w", uploadID, err)
		}
		if upload == nil {
			slog.WarnContext(ctx, "upload not found (skipped)", "noteID", simpleNote.ID, "uploadID", uploadID)
			continue
		}
		if !upload.IsLocal {
			slog.WarnContext(ctx, "found remote upload in local note (skipped)", "noteID", simpleNote.ID, "uploadID", uploadID)
			continue
		}
		apubUploads = append(apubUploads, types.ApubUpload{
			MimeType: upload.MimeType,
			URL:      s.urlForLocalUpload(uploadID),
		})
	}

	var replyToURL string
	if simpleNote.ReplyToID != nil {
		replyToURL, err = s.findApubURLByNoteID(ctx, *simpleNote.ReplyToID, true)
		if err != nil {
			return types.ApubNote{}, fmt.Errorf("error fetching reply-to note %s: %w", *simpleNote.ReplyToID, err)
		}
	}

	addresses, err := s.calculateToAndCc(ctx, simpleNote.ID, true)
	if err != nil {
		return types.ApubNote{}, fmt.Errorf("failed to calculate to/cc: %w", err)
	}

	return types.ApubNote{
		Basic: *simpleNote,
		Apub: types.ApubNoteData{
			AuthorURL: authorURL,

			To:      addresses.To,
			Cc:      addresses.Cc,
			Inboxes: addresses.Inboxes,

			Hashtags:   apubHashtags,
			Mentions:   apubMentions,
			Uploads:    apubUploads,
			ReplyToURL: replyToURL,

			URL:     url,
			ViewURL: viewURL,
		},
	}, nil
}

func (s *State) CheckNoteVisibility(ctx context.Context, viewerID *types.UserID, note types.SimpleNote) (bool, error) {
	if viewerID != nil && *viewerID == note.Author.ID {
		return true, nil
	}

	switch note.Visibility {
	case types.NoteVisibilityPublic:
		fallthrough
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

	panic(fmt.Sprintf("unknown note visibility: %s", note.Visibility))
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

	rawNote, err := s.findNoteByIDRaw(ctx, note.ID, false)
	if err != nil {
		return details, err
	}

	if viewerID != nil {
		details.Renoted, err = s.checkNoteRenoted(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}

		reacted, err := s.checkNoteReacted(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}
		if reacted != nil {
			details.Reacted = *reacted
		}

		details.Bookmarked, err = s.checkNoteBookmarked(ctx, note.ID, *viewerID)
		if err != nil {
			return details, err
		}

		details.IsMyNote = *viewerID == note.Author.ID
	}

	var (
		replyCount       int64
		renoteCount      int64
		reactionCountRaw []struct {
			Reaction string `db:"reaction"`
			Count    int64  `db:"count"`
		}
	)
	if err := s.DB(ctx).Model(&models.Note{}).Where("reply_to_id = ? AND deleted_at IS NULL", note.ID).Count(&replyCount).Error; err != nil {
		return details, err
	}
	if err := s.DB(ctx).Model(&models.Note{}).Where("renote_of_id = ? AND content IS NULL AND deleted_at IS NULL", note.ID).Count(&renoteCount).Error; err != nil {
		return details, err
	}
	if err := s.DB(ctx).Model(&models.NoteReaction{}).Select("reaction, COUNT(*) AS count").Where("note_id = ?", note.ID).Group("reaction").Scan(&reactionCountRaw).Error; err != nil {
		return details, err
	}
	reactionCount := make(map[string]uint64, len(reactionCountRaw))
	for _, raw := range reactionCountRaw {
		reactionCount[raw.Reaction] = uint64(raw.Count)
	}
	details.ReplyCount = uint64(replyCount)
	details.RenoteCount = uint64(renoteCount)
	details.ReactionCount = reactionCount

	details.Hashtags, err = s.getNoteHashtags(ctx, note.ID)
	if err != nil {
		return details, err
	}
	details.Mentions, err = s.getNoteMentions(ctx, note.ID)
	if err != nil {
		return details, err
	}

	details.RemoteViewURL = sqlToString(rawNote.Note.ViewURL)

	return details, nil
}

func (s *State) checkNoteMentioned(ctx context.Context, noteID types.NoteID,
	targetUserID types.UserID) (bool, error) {
	var mentions models.NoteMention
	if err := s.DB(ctx).Where("note_id = ? AND target_user_id = ?", noteID, targetUserID).First(&mentions).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

func (s *State) checkNoteRenoted(ctx context.Context, noteID types.NoteID, renoterID types.UserID) (bool, error) {
	var count int64
	if err := s.DB(ctx).Where("renote_of_id = ? AND author_id = ? AND content IS NULL", noteID, renoterID).Model(&models.Note{}).Count(&count).Error; err != nil {
		return false, err
	}
	return count > 0, nil
}

func (s *State) getNoteHashtags(ctx context.Context, noteID types.NoteID) ([]string, error) {
	var hashtags []models.NoteTag
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
	var mentions []models.NoteMention
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

func (s *State) GetNoteRenoters(ctx context.Context, viewerID *types.UserID, noteID types.NoteID, limit int, page int) ([]types.SimpleUser, error) {
	// visibility check
	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, viewerID, noteID)
	if err != nil {
		return nil, err
	}
	if note == nil {
		return nil, ErrNoteNotFound
	}

	var renotes []models.Note
	if err := s.DB(ctx).Model(&models.Note{}).Where("renote_of_id = ? AND content IS NULL", noteID).Find(&renotes).Error; err != nil {
		return nil, err
	}

	renoters := make([]types.SimpleUser, len(renotes))
	for i, renote := range renotes {
		renoter, err := s.FindUserByID(ctx, renote.AuthorID)
		if err != nil {
			return nil, err
		}
		if renoter == nil {
			return nil, fmt.Errorf("renoter not found: %s", renote.AuthorID)
		}
		renoters[i] = *renoter
	}

	return renoters, nil
}

func (s *State) GetMentions(ctx context.Context, viewerID *types.UserID, noteID types.NoteID) ([]types.SimpleUser, error) {
	// visibility check
	note, err := s.FindNoteByIDWithVisibilityCheck(ctx, viewerID, noteID)
	if err != nil {
		return nil, err
	}
	if note == nil {
		return nil, ErrNoteNotFound
	}

	var noteMentions []models.NoteMention
	if err := s.DB(ctx).Model(&models.NoteMention{}).Where("note_id = ?", noteID).Find(&noteMentions).Error; err != nil {
		return nil, err
	}

	mentionUserIDs := make([]types.SimpleUser, len(noteMentions))
	for i, noteMention := range noteMentions {
		mention, err := s.FindUserByID(ctx, noteMention.TargetUserID)
		if err != nil {
			return nil, err
		}
		if mention == nil {
			return nil, fmt.Errorf("mention not found: %s", noteMention.TargetUserID)
		}
		mentionUserIDs[i] = *mention
	}

	return mentionUserIDs, nil
}

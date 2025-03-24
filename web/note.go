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

package web

import (
	"net/http"
	"net/url"
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/service"
	"github.com/lightpub-dev/lightpub/types"
)

const (
	hxNoteRefreshEvent = "note-refresh"

	trendShowCount = 5

	maxInMemoryBytes = 1024 * 1024 * 1 // 1 MB
)

type ClientCreateNoteParams struct {
	Authed    bool
	Title     string
	ReplyToID *string
}

type ClientNoteParams struct {
	Note       *types.DetailedNote
	RenoteInfo *ClientRenoteInfo
	Reply      ClientCreateNoteParams
	Authed     bool
}

type ClientRenoteInfo struct {
	User types.SimpleUser
}

type ClientNotesParams struct {
	Data    []ClientNoteParams
	NextURL *string
}

func (s *State) renderNote(note *types.DetailedNote, renoter *types.SimpleUser, authed bool) ClientNoteParams {
	var renoteInfo *ClientRenoteInfo
	if renoter != nil {
		renoteInfo = &ClientRenoteInfo{
			User: *renoter,
		}
	}

	noteID := note.Basic.ID.String()
	return ClientNoteParams{
		Note:       note,
		RenoteInfo: renoteInfo,
		Reply: ClientCreateNoteParams{
			Title:     "返信",
			Authed:    authed,
			ReplyToID: &noteID,
		},
		Authed: authed,
	}
}

func (s *State) renderNotes(notes []types.DetailedNote, authed bool, nextURL *string) ClientNotesParams {
	data := make([]ClientNoteParams, 0, len(notes))
	for _, note := range notes {
		data = append(data, s.renderNote(&note, nil, authed))
	}

	return ClientNotesParams{
		Data:    data,
		NextURL: nextURL,
	}
}

func (s *State) GetNote(c echo.Context) error {
	var param struct {
		ID        string `param:"id"`
		RenotedBy string `query:"renotedBy"`
	}
	if err := c.Bind(&param); err != nil {
		return errBadInput
	}

	noteID, err := types.ParseNoteID(param.ID)
	if err != nil {
		return errBadInput
	}
	var renotedByID *types.UserID
	if param.RenotedBy != "" {
		renotedByIDP, err := types.ParseUserID(param.RenotedBy)
		if err != nil {
			return errBadInput
		}
		renotedByID = &renotedByIDP
	}

	viewerID := getViewerID(c)
	note, err := s.service.FindNoteByIDWithDetails(c.Request().Context(), viewerID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return failure.NewError(http.StatusNotFound, "note not found")
	}

	var renoteUser *types.SimpleUser
	if renotedByID != nil {
		renoteUserP, err := s.service.FindUserByID(c.Request().Context(), *renotedByID)
		if err != nil {
			return err
		}
		if renoteUserP == nil {
			return failure.NewError(http.StatusNotFound, "renote user not found")
		}
		renoteUser = renoteUserP
	}

	renderParams := s.renderNote(note, renoteUser, viewerID != nil)

	// private: for enforcing access control
	c.Response().Header().Set(cacheControl, "private, max-age=60, stale-while-revalidate=86400")
	return c.Render(http.StatusOK, "note.html", renderParams)
}

func (s *State) DeleteNote(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	if err := s.service.DeleteNoteByID(c.Request().Context(), *viewerID, noteID); err != nil {
		return err
	}

	c.Response().Header().Set(hxRefresh, trueHeaderValue)
	return c.NoContent(http.StatusOK)
}

func (s *State) CreateNote(c echo.Context) error {
	if err := c.Request().ParseMultipartForm(maxInMemoryBytes); err != nil {
		return err
	}

	content := c.FormValue("content")
	contentTypeStr := c.FormValue("contentType")
	sensitive := c.FormValue("sensitive") == "on"
	visibilityStr := c.FormValue("visibility")
	replyToIDStr := c.FormValue("replyToId")
	tags := c.Request().PostForm["tags[]"]
	mentions := c.Request().PostForm["mentions[]"]

	if !types.IsValidContentType(contentTypeStr) {
		return errBadInput
	}
	contentType := types.NoteContentType(contentTypeStr)
	if contentType == types.NoteContentTypeHTML {
		// cannot create HTML note from web
		return errBadInput
	}

	if !types.IsValidVisibility(visibilityStr) {
		return errBadInput
	}
	visibility := types.NoteVisibility(visibilityStr)

	var replyToID *types.NoteID
	if replyToIDStr != "" {
		replyToIDP, err := types.ParseNoteID(replyToIDStr)
		if err != nil {
			return errBadInput
		}
		replyToID = &replyToIDP
	}

	form, err := c.MultipartForm()
	if err != nil {
		return err
	}
	files := form.File["file"]
	uploadIDs := make([]types.UploadID, 0, len(files))
	for _, file := range files {
		uploadID, err := s.newUpload(c.Request().Context(), file)
		if err != nil {
			return err
		}
		uploadIDs = append(uploadIDs, uploadID)
	}

	mentionUserIDs := make([]types.UserID, 0, len(mentions))
	for _, mention := range mentions {
		specifier, ok := types.ParseUserSpecifier(mention, s.MyDomain())
		if !ok {
			return failure.NewError(http.StatusBadRequest, "bad mention: "+mention)
		}
		mentionUserID, err := s.service.FindLocalUserIDBySpecifier(c.Request().Context(), specifier)
		if err != nil {
			return err
		}
		if mentionUserID == nil {
			return failure.NewError(http.StatusNotFound, "mention user not found")
		}
		mentionUserIDs = append(mentionUserIDs, *mentionUserID)
	}

	viewerID := getViewerID(c) // must be non-nil
	noteContent := types.NoteContent{
		Type: contentType,
		Data: content,
	}

	noteID, err := s.service.CreateNote(c.Request().Context(), *viewerID, service.CreateNoteParams{
		Content:    noteContent,
		Visibility: &visibility,
		ReplyToID:  replyToID,
		Uploads:    uploadIDs,
		Sensitive:  sensitive,
		Hashtags:   tags,
		Mentions:   mentionUserIDs,
	})
	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRefresh, trueHeaderValue)
	return c.JSON(http.StatusOK, map[string]interface{}{
		"note_id": noteID,
	})
}

func (s *State) CreateRenote(c echo.Context) error {
	var param struct {
		Visibility types.NoteVisibility `json:"visibility" validate:"required,oneof=public unlisted follower private"`
		NoteIDStr  string               `param:"id" validate:"required"`
	}
	if err := c.Bind(&param); err != nil {
		return errBadInput
	}
	if err := validate.Struct(param); err != nil {
		return errBadInput
	}

	noteID, err := types.ParseNoteID(param.NoteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	renoteID, err := s.service.CreateRenote(c.Request().Context(), *viewerID, noteID, param.Visibility)
	if err != nil {
		return err
	}

	return c.JSON(http.StatusOK, map[string]interface{}{
		"note_id": renoteID,
	})
}

type NoteEditParams struct {
	Note *types.DetailedNote
}

func (s *State) GetEditNotePage(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c) // non-nil

	note, err := s.service.FindNoteByIDWithDetails(c.Request().Context(), viewerID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return failure.NewError(http.StatusNotFound, "note not found")
	}

	if note.Basic.Author.ID != *viewerID {
		return failure.NewError(http.StatusForbidden, "you are not the author of this note")
	}

	// always returns the latest version of the note
	c.Response().Header().Set(cacheControl, "no-store")
	return c.Render(http.StatusOK, "editNote.html", NoteEditParams{
		Note: note,
	})
}

func (s *State) PatchNote(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c) // non-nil

	content := c.FormValue("content")
	contentTypeStr := c.FormValue("contentType")
	if !types.IsValidContentType(contentTypeStr) {
		return errBadInput
	}
	contentType := types.NoteContentType(contentTypeStr)
	if contentType == types.NoteContentTypeHTML {
		// cannot create HTML note from web
		return errBadInput
	}

	err = s.service.EditNote(c.Request().Context(), *viewerID, noteID, service.NoteEditParams{
		Content: types.NoteContent{
			Type: contentType,
			Data: content,
		},
	})
	if err != nil {
		return err
	}

	return c.Redirect(http.StatusSeeOther, "/note/"+noteIDStr)
}

func (s *State) PutBookmarkOnNote(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	if err := s.service.NoteBookmarkAdd(c.Request().Context(), *viewerID, noteID); err != nil {
		return err
	}

	c.Response().Header().Set(hxTrigger, hxNoteRefreshEvent)
	return c.NoContent(http.StatusOK)
}

func (s *State) DeleteBookmarkOnNote(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	if err := s.service.NoteBookmarkRemove(c.Request().Context(), *viewerID, noteID); err != nil {
		return err
	}

	c.Response().Header().Set(hxTrigger, hxNoteRefreshEvent)
	return c.NoContent(http.StatusOK)
}

func (s *State) GetNoteReplies(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return err
	}

	var query struct {
		BeforeTime *time.Time `query:"beforeTime"`
	}
	if err := c.Bind(&query); err != nil {
		return err
	}

	viewerID := getViewerID(c)

	notes, err := s.service.GetNoteReplies(c.Request().Context(), viewerID, noteID, paginationSizeP1, query.BeforeTime)
	if err != nil {
		return err
	}

	var nextURL *string
	if len(notes) == paginationSizeP1 {
		beforeTime := notes[len(notes)-1].Basic.CreatedAt.UTC().Format(time.RFC3339Nano)
		nextURLP := buildURLWithParams(c.Request().URL, map[string]string{
			"beforeTime": beforeTime,
		})
		nextURL = &nextURLP
		notes = notes[:paginationSize]
	}

	renderParams := s.renderNotes(notes, viewerID != nil, nextURL)

	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "notes.html", renderParams)
}

func (s *State) GetTimeline(c echo.Context) error {
	var query struct {
		BeforeTime *time.Time `query:"beforeTime"`
		Public     bool       `query:"public"`
	}
	if err := c.Bind(&query); err != nil {
		return err
	}

	viewerID := getViewerID(c)
	if viewerID == nil && !query.Public {
		return failure.NewError(http.StatusUnauthorized, "you have to log in to see non-public timeline")
	}

	var (
		notes []types.DetailedNote
		err   error
	)
	if query.Public {
		notes, err = s.service.GetPublicTimeline(c.Request().Context(), viewerID, paginationSizeP1, query.BeforeTime)
	} else {
		notes, err = s.service.GetTimeline(c.Request().Context(), *viewerID, paginationSizeP1, query.BeforeTime)
	}
	if err != nil {
		return err
	}

	var nextURL *string
	if len(notes) == paginationSizeP1 {
		beforeTime := notes[len(notes)-1].Basic.CreatedAt.UTC().Format(time.RFC3339Nano)
		nextURLP := buildURLWithParams(c.Request().URL, map[string]string{
			"beforeTime": beforeTime,
		})
		nextURL = &nextURLP
		notes = notes[:paginationSize]
	}

	renderParams := s.renderNotes(notes, viewerID != nil, nextURL)
	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "notes.html", renderParams)
}

func (s *State) GetUserNoteList(c echo.Context) error {
	var query struct {
		BeforeTime *time.Time `query:"beforeTime"`
	}
	if err := c.Bind(&query); err != nil {
		return err
	}

	// Get the user ID from URL parameter
	userIDStr := c.Param("id")
	userID, err := types.ParseUserID(userIDStr)
	if err != nil {
		return failure.NewError(http.StatusBadRequest, "invalid user ID")
	}

	// Get current viewer ID (if logged in)
	viewerID := getViewerID(c)

	// Get user notes with pagination
	notes, err := s.service.GetUserNotes(c.Request().Context(), viewerID, userID, paginationSizeP1, query.BeforeTime)
	if err != nil {
		return err
	}

	// Handle pagination
	var nextURL *string
	if len(notes) == paginationSizeP1 {
		beforeTime := notes[len(notes)-1].Basic.CreatedAt.UTC().Format(time.RFC3339Nano)
		nextURLP := buildURLWithParams(c.Request().URL, map[string]string{
			"beforeTime": beforeTime,
		})
		nextURL = &nextURLP
		notes = notes[:paginationSize]
	}

	// Render the notes template
	renderParams := s.renderNotes(notes, viewerID != nil, nextURL)

	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "notes.html", renderParams)
}

type TrendRenderParams struct {
	Data []TrendEntry
}

type TrendEntry struct {
	service.TrendEntry
	URL string
}

func (s *State) GetTrends(c echo.Context) error {
	trends, err := s.service.GetTrendingTags(c.Request().Context(), trendShowCount)
	if err != nil {
		return err
	}

	entries := make([]TrendEntry, 0, len(trends))
	for _, trend := range trends {
		entries = append(entries, TrendEntry{
			TrendEntry: trend,
			URL:        "/timeline?tag=" + url.QueryEscape(trend.Hashtag),
		})
	}

	c.Response().Header().Set(cacheControl, "public, max-age=60") // cache for 1 minute
	return c.Render(http.StatusOK, "trends.html", TrendRenderParams{
		Data: entries,
	})
}

type NoteDetailsParams struct {
	Og     NoteOpenGraph
	Create ClientCreateNoteParams
	Note   ClientNoteParams
}

type NoteOpenGraph struct {
	URL            string
	AuthorID       string
	AuthorNickname string
	NoteContent    string
	BaseURL        string
}

func (s *State) ClientGetNote(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	note, err := s.service.FindNoteByIDWithDetails(c.Request().Context(), viewerID, noteID)
	if err != nil {
		return err
	}
	if note == nil {
		return failure.NewError(http.StatusNotFound, "note not found")
	}

	og := NoteOpenGraph{
		URL:            s.BaseURL().JoinPath("client", "note", noteIDStr).String(),
		AuthorID:       note.Basic.Author.ID.String(),
		AuthorNickname: note.Basic.Author.Nickname,
		NoteContent:    note.Basic.Content.Data,
		BaseURL:        s.BaseURL().String(),
	}
	create := ClientCreateNoteParams{
		Authed: viewerID != nil,
		Title:  "ノート作成",
	}

	c.Response().Header().Set(cacheControl, "private, max-age=60, stale-while-revalidate=86400")
	return c.Render(http.StatusOK, "topNoteDetails.html", NoteDetailsParams{
		Og:     og,
		Create: create,
		Note: ClientNoteParams{
			Note:       note,
			RenoteInfo: nil,
			Reply: ClientCreateNoteParams{
				Title:     "返信",
				Authed:    viewerID != nil,
				ReplyToID: &noteIDStr,
			},
			Authed: viewerID != nil,
		},
	})
}

type ModalUserListParams struct {
	Title   string
	Data    []UserListEntry
	NextURL string
}

func (s *State) GetRenotersModal(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	var p struct {
		Page int `query:"page"`
	}
	if err := c.Bind(&p); err != nil {
		return errBadInput
	}
	if p.Page < 0 {
		return errBadInput
	}

	viewerID := getViewerID(c)

	renoters, err := s.service.GetNoteRenoters(c.Request().Context(), viewerID, noteID, paginationSize, p.Page)
	if err != nil {
		return err
	}

	var nextURL string
	if len(renoters) == paginationSize {
		nextURL = buildURLWithParams(c.Request().URL, map[string]string{
			"page": strconv.Itoa(p.Page + 1),
		})
	}

	entries := make([]UserListEntry, 0, len(renoters))
	for _, r := range renoters {
		entries = append(entries, UserListEntry{
			User:      r,
			CreatedAt: nil,
		})
	}

	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "modalUserList.html", &ModalUserListParams{
		Title:   "リノート一覧",
		Data:    entries,
		NextURL: nextURL,
	})
}

func (s *State) GetNoteMentionsModal(c echo.Context) error {
	noteIDStr := c.Param("id")
	noteID, err := types.ParseNoteID(noteIDStr)
	if err != nil {
		return errBadInput
	}

	viewerID := getViewerID(c)

	mentions, err := s.service.GetMentions(c.Request().Context(), viewerID, noteID)
	if err != nil {
		return err
	}

	entries := make([]UserListEntry, 0, len(mentions))
	for _, r := range mentions {
		entries = append(entries, UserListEntry{
			User:      r,
			CreatedAt: nil,
		})
	}

	c.Response().Header().Set(cacheControl, "private, no-cache")
	return c.Render(http.StatusOK, "modalUserList.html", &ModalUserListParams{
		Title:   "メンション一覧",
		Data:    entries,
		NextURL: "",
	})
}

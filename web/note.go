package web

import (
	"io"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/service"
	"github.com/lightpub-dev/lightpub/types"
)

const (
	hxNoteRefreshEvent = "note-refresh"
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
		RenotedBy string `query:"renoted_by"`
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
	content := c.FormValue("content")
	contentTypeStr := c.FormValue("contentType")
	sensitive := c.FormValue("sensitive") == "on"
	visibilityStr := c.FormValue("visibility")
	replyToIDStr := c.FormValue("replyToId")

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
		if !strings.HasPrefix(file.Header.Get("Content-Type"), "image/") {
			return failure.NewError(http.StatusBadRequest, "invalid file type")
		}

		src, err := file.Open()
		if err != nil {
			return err
		}
		defer src.Close()

		// copy to tempfile
		tmp, err := os.CreateTemp("", "lp-upload-")
		if err != nil {
			return err
		}
		defer os.Remove(tmp.Name())
		defer tmp.Close()

		if _, err := io.Copy(tmp, src); err != nil {
			return err
		}

		uploadID, err := s.service.UploadFile(c.Request().Context(), tmp.Name())
		if err != nil {
			return err
		}
		uploadIDs = append(uploadIDs, uploadID)
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
	})
	if err != nil {
		return err
	}

	return c.JSON(http.StatusOK, map[string]interface{}{
		"note_id": noteID,
	})
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

func (s *State) GetTimeline(c echo.Context) error {
	var query struct {
		BeforeTime *time.Time `query:"before_time"`
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
			"before_time": beforeTime,
		})
		nextURL = &nextURLP
		notes = notes[:paginationSize]
	}

	renderParams := s.renderNotes(notes, viewerID != nil, nextURL)
	return c.Render(http.StatusOK, "notes.html", renderParams)
}

package web

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/types"
)

type ClientCreateNoteParams struct {
	Authed    bool
	Title     string
	ReplyToID *string
}

type ClientNoteParams struct {
	Note       *types.DetailedNote
	RenoteInfo *ClientRenoteInfo
	Authed     bool
}

type ClientRenoteInfo struct {
	User types.SimpleUser
}

func (s *State) renderNote(note *types.DetailedNote, renoter *types.SimpleUser, authed bool) ClientNoteParams {
	var renoteInfo *ClientRenoteInfo
	if renoter != nil {
		renoteInfo = &ClientRenoteInfo{
			User: *renoter,
		}
	}

	return ClientNoteParams{
		Note:       note,
		RenoteInfo: renoteInfo,
		Authed:     authed,
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

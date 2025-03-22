package web

import (
	"io"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/types"
)

func (s *State) GetUpload(c echo.Context) error {
	uploadIDStr := c.Param("id")
	uploadID, err := types.ParseUploadID(uploadIDStr)
	if err != nil {
		return err
	}

	upload, err := s.service.GetUpload(c.Request().Context(), uploadID)
	if err != nil {
		return err
	}

	bytes, err := io.ReadAll(upload)
	if err != nil {
		return err
	}

	return c.Blob(http.StatusOK, upload.MimeType, bytes)
}

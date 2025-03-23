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
	"context"
	"io"
	"mime/multipart"
	"net/http"
	"os"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
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

func (s *State) newUpload(ctx context.Context, file *multipart.FileHeader) (types.UploadID, error) {
	if !strings.HasPrefix(file.Header.Get("Content-Type"), "image/") {
		return types.UploadID{}, failure.NewError(http.StatusBadRequest, "invalid file type")
	}

	src, err := file.Open()
	if err != nil {
		return types.UploadID{}, err
	}
	defer src.Close()

	// copy to tempfile
	tmp, err := os.CreateTemp("", "lp-upload-")
	if err != nil {
		return types.UploadID{}, err
	}
	defer os.Remove(tmp.Name())
	defer tmp.Close()

	if _, err := io.Copy(tmp, src); err != nil {
		return types.UploadID{}, err
	}

	uploadID, err := s.service.UploadFile(ctx, tmp.Name())
	if err != nil {
		return types.UploadID{}, err
	}
	return uploadID, nil
}

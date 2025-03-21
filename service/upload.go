package service

import (
	"context"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/gabriel-vasile/mimetype"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/service/upload"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
	"resty.dev/v3"
)

var (
	ErrNoUploadFound     = NewServiceError(404, "upload not found")
	ErrInvalidUploadInDB = NewInternalServerError("invalid upload in database")
)

type UploadError struct {
	Message string
	Status  int
}

func (e UploadError) Error() string {
	return e.Message
}

type uploadFileInfo struct {
	UploadID types.UploadID
	Filename string
	MimeType string
}

func (s *State) UploadFile(ctx context.Context, tempFilePath string) (types.UploadID, error) {
	uploadInfo, err := s.saveUploadFile(tempFilePath)
	if err != nil {
		return types.UploadID{}, err
	}

	err = s.saveUploadFileInfo(ctx, uploadInfo)
	if err != nil {
		return types.UploadID{}, err
	}

	return uploadInfo.UploadID, nil
}

func (s *State) saveUploadFile(tempFilePath string) (uploadFileInfo, error) {
	uploadID := types.NewUploadID()

	// Get file type
	mime, err := mimetype.DetectFile(tempFilePath)
	if err != nil {
		return uploadFileInfo{}, err
	}

	ext := ""
	if mime != nil {
		ext = "." + mime.Extension()
	}

	filename := uploadID.String() + ext
	uploadPath := filepath.Join(s.getUploadsDir(), filename)

	// Copy temp file to upload path
	source, err := os.Open(tempFilePath)
	if err != nil {
		return uploadFileInfo{}, err
	}
	defer source.Close()

	destination, err := os.Create(uploadPath)
	if err != nil {
		return uploadFileInfo{}, err
	}
	defer destination.Close()

	_, err = io.Copy(destination, source)
	if err != nil {
		return uploadFileInfo{}, err
	}

	// Verify it's an image
	if mime != nil && !isImage(mime.String()) {
		return uploadFileInfo{}, UploadError{Message: "invalid file type", Status: http.StatusBadRequest}
	}

	// Remove EXIF data
	err = removeExif(uploadPath)
	if err != nil {
		return uploadFileInfo{}, err
	}

	return uploadFileInfo{
		UploadID: uploadID,
		Filename: filename,
		MimeType: mime.String(),
	}, nil
}

func isImage(mimeType string) bool {
	return strings.HasPrefix(mimeType, "image/")
}

func removeExif(uploadPath string) error {
	return upload.RemoveGPSData(uploadPath)
}

func (s *State) saveUploadFileInfo(ctx context.Context, info uploadFileInfo) error {
	return s.DB(ctx).Create(&db.Upload{
		ID:       info.UploadID,
		Filename: stringToSql(info.Filename),
		MimeType: info.MimeType,
	}).Error
}

type UploadResult struct {
	IsLocal  bool
	MimeType string

	// If Local:
	RelativePath string

	// If Remote:
	CacheControl string
	Response     []byte
	ContentType  string
	StatusCode   int
}

func (s *State) GetUpload(ctx context.Context, uploadID string, client *resty.Client) (UploadResult, error) {
	var upload db.Upload
	err := s.DB(ctx).Where("id = ?", uploadID).First(&upload).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return UploadResult{}, ErrNoUploadFound
		}
		return UploadResult{}, err
	}

	// For demonstration, just showing the structure
	var result UploadResult

	if (upload.Filename.Valid && upload.URL.Valid) || (!upload.Filename.Valid && !upload.URL.Valid) {
		return UploadResult{}, ErrInvalidUploadInDB
	}

	// If local file:
	if upload.Filename.Valid {
		result.IsLocal = true
		result.RelativePath = "example.jpg"
		result.MimeType = "image/jpeg"
	}

	// If remote file:
	if upload.URL.Valid {
		result, err = s.handleRemoteUpload(ctx, upload.URL.String)
		if err != nil {
			return UploadResult{}, err
		}
	}

	return result, nil
}

func (s *State) handleRemoteUpload(ctx context.Context, uploadURL string) (UploadResult, error) {
	var result UploadResult
	result.IsLocal = false

	resp, err := s.uploadFetchClient.R().WithContext(ctx).Get(uploadURL)
	if err != nil {
		return result, err
	}

	if resp.Header().Get("Cache-Control") != "" {
		result.CacheControl = resp.Header().Get("Cache-Control")
	}

	result.StatusCode = resp.StatusCode()
	if result.StatusCode != http.StatusOK {
		return result, nil
	}

	contentType := resp.Header().Get("Content-Type")
	if contentType == "" {
		return result, errors.New("invalid remote: no content type")
	}
	result.ContentType = contentType

	result.Response = resp.Bytes()

	return result, nil
}

func (s *State) registerRemoteUpload(ctx context.Context, url string) (types.UploadID, error) {
	mimeType, err := s.checkRemoteMimeType(ctx, url)
	if err != nil {
		return types.UploadID{}, err
	}

	uploadID := types.NewUploadID()

	err = s.DB(ctx).Create(&db.Upload{
		ID:       uploadID,
		URL:      stringToSql(url),
		MimeType: mimeType,
	}).Error
	if err != nil {
		return types.UploadID{}, err
	}

	return uploadID, nil
}

func (s *State) checkRemoteMimeType(ctx context.Context, url string) (string, error) {
	resp, err := s.uploadFetchClient.R().WithContext(ctx).Head(url)
	if err != nil {
		return "", UploadError{Message: "Failed to send request to the URL", Status: http.StatusBadGateway}
	}

	contentType := resp.Header().Get("Content-Type")
	if contentType == "" {
		return "", UploadError{Message: "No content-type header found", Status: http.StatusBadRequest}
	}

	return contentType, nil
}

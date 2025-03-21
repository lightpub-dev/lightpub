package upload

import (
	"os"
	"os/exec"
	"path/filepath"

	"github.com/lightpub-dev/lightpub/failure"
)

var (
	ErrFileNotFound = failure.NewInternalServerError("upload file not found, failed to remove GPS data")
)

func RemoveGPSData(filename string) error {
	// Check if file exists
	if _, err := os.Stat(filename); os.IsNotExist(err) {
		return ErrFileNotFound
	}

	// Get absolute path to ensure exiftool can find the file
	absPath, err := filepath.Abs(filename)
	if err != nil {
		return failure.NewInternalServerErrorWithCause("failed to get absolute path", err)
	}

	// Prepare the exiftool command
	cmd := exec.Command("exiftool", "-gps:all=", "-xmp:geotag=", absPath)

	// Execute the command
	_, err = cmd.CombinedOutput()
	if err != nil {
		return failure.NewInternalServerErrorWithCause("failed to remove GPS data", err)
	}

	return nil
}

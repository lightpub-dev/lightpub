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

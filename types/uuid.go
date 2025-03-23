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

package types

import (
	"database/sql/driver"
	"fmt"

	"github.com/google/uuid"
)

// BinUUID is a custom UUID type which stores value as binary in database
type BinUUID struct {
	inner uuid.UUID
}

func (u BinUUID) String() string {
	return u.inner.String()
}

func ParseBinUUID(s string) (BinUUID, error) {
	id, err := uuid.Parse(s)
	return BinUUID{id}, err
}

func WrapBinUUID(u uuid.UUID) BinUUID {
	return BinUUID{u}
}

func (u *BinUUID) Scan(value interface{}) error {
	if value == nil {
		return nil
	}

	switch value := value.(type) {
	case []byte:
		var err error
		u.inner, err = uuid.FromBytes(value)
		if err != nil {
			return err
		}
	default:
		return fmt.Errorf("invalid BinUUID value: %v", value)
	}
	return nil
}

func (u BinUUID) Value() (driver.Value, error) {
	b := [16]byte(u.inner)
	return b[:], nil
}

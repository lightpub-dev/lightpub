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

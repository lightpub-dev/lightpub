package db

import (
	"database/sql/driver"
	"fmt"
	"strings"

	"github.com/google/uuid"
)

type UUID uuid.UUID

func ParseTo(u *UUID, s string) error {
	parsed, err := uuid.Parse(s)
	if err != nil {
		return err
	}
	*u = UUID(parsed)
	return nil
}

func UuidToString(u uuid.UUID) string {
	return strings.Replace(u.String(), "-", "", -1)
}

func (us *UUID) Scan(dbValue interface{}) error {
	switch value := dbValue.(type) {
	case []byte:
		u, err := uuid.Parse(string(value))
		if err != nil {
			return err
		}
		*us = UUID(u)
		return nil
	default:
		return fmt.Errorf("unsupported type for UUIDString: %T", dbValue)
	}
}

func (us UUID) Value() (driver.Value, error) {
	return UuidToString(uuid.UUID(us)), nil
}

func (UUID) GormDataType() string {
	return "VARCHAR(32)"
}

func (us UUID) String() string {
	return strings.ReplaceAll(uuid.UUID(us).String(), "-", "")
}

func (us UUID) AsNullable() NullUUID {
	return NullUUID{
		UUID:  us,
		Valid: true,
	}
}

type NullUUID struct {
	UUID  UUID
	Valid bool
}

func (us *NullUUID) Scan(dbValue interface{}) error {
	if dbValue == nil {
		us.UUID, us.Valid = UUID{}, false
		return nil
	}

	switch value := dbValue.(type) {
	case []byte:
		u, err := uuid.Parse(string(value))
		if err != nil {
			return err
		}
		us.UUID = UUID(u)
		us.Valid = true
		return nil
	default:
		return fmt.Errorf("unsupported type for UUIDString: %T", dbValue)
	}
}

func (us NullUUID) Value() (driver.Value, error) {
	if !us.Valid {
		return nil, nil
	}

	return us.UUID.Value()
}

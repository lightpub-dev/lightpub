package db

import (
	"database/sql/driver"
	"fmt"

	"github.com/google/uuid"
)

type UUID uuid.UUID

func (us *UUID) Scan(dbValue interface{}) error {
	switch value := dbValue.(type) {
	case []byte:
		u, err := uuid.FromBytes(value)
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
	bs := [16]byte(uuid.UUID(us))
	slice := bs[:]
	return slice, nil
}

func (UUID) GormDataType() string {
	return "BINARY(16)"
}

func (us UUID) String() string {
	return uuid.UUID(us).String()
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
		u, err := uuid.FromBytes(value)
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

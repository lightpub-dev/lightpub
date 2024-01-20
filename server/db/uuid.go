package db

import (
	"context"
	"fmt"
	"reflect"

	"github.com/google/uuid"
	"gorm.io/gorm/schema"
)

type UUID uuid.UUID

func (us *UUID) Scan(ctx context.Context, field *schema.Field, dst reflect.Value, dbValue interface{}) error {
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

// func (us *UUID) Value(ctx context.Context, field *schema.Field, dst reflect.Value, fieldValue interface{}) (interface{}, error) {
// 	if us == nil {
// 		return nil, nil
// 	}

// 	bs := [16]byte(uuid.UUID(*us))
// 	return bs[:], nil
// }

func (us UUID) Value(ctx context.Context, field *schema.Field, dst reflect.Value, fieldValue interface{}) (interface{}, error) {
	bs := [16]byte(uuid.UUID(us))
	return bs[:], nil
}

func (UUID) GormDataType() string {
	return "BINARY(16)"
}

func (us UUID) String() string {
	return uuid.UUID(us).String()
}

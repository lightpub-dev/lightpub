package apub

import (
	"encoding/json"
	"fmt"
	"reflect"
	"strings"
)

var (
	appContext = []any{
		"https://www.w3.org/ns/activitystreams",
	}
)

type WithContext[T Signable] struct {
	Data T
}

func (w WithContext[T]) Signer() Actor {
	return w.Data.Signer()
}

func (w WithContext[T]) MarshalJSON() ([]byte, error) {
	fields := make(map[string]any)

	v := reflect.ValueOf(w.Data)
	if v.Kind() == reflect.Ptr || v.Kind() == reflect.Interface {
		v = v.Elem()
	}
	if v.Kind() != reflect.Struct {
		return nil, fmt.Errorf("WithContext: expected struct, got %T", w.Data)
	}

	for i := 0; i < v.NumField(); i++ {
		var key string
		rawKey := v.Type().Field(i).Tag.Get("json")
		// handle ignore
		if rawKey == "-" {
			continue
		}
		// handle omitempty
		if strings.HasSuffix(rawKey, ",omitempty") {
			key = strings.TrimSuffix(rawKey, ",omitempty")
			if v.Field(i).IsZero() {
				continue
			}
		} else {
			key = rawKey
		}
		if key == "" {
			key = v.Type().Field(i).Name
		}

		fields[key] = v.Field(i).Interface()
	}

	fields["@context"] = appContext

	return json.Marshal(fields)
}

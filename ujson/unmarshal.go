// ujson provides helper for unmarshalling untagged-union JSON data into Go structs.
// Example:
/*
	type MyUnionData struct {
		Variant string `ujson:"-"`

		Kind1 string `ujson:"kind1"`
		Kind2 int `ujson:"kind2"`
	}

	func (d *MyUnionData) UnmarshalJSON(data []byte) error {
		return ujson.UntaggedUnmarshalJSON(data, d)
	}

	type Data struct {
		ID string `json:"id"`
		Data MyUnion `json:"data"`
	}
*/

package ujson

import (
	"encoding/json"
	"errors"
	"fmt"
	"reflect"
)

const (
	variantTag = "-"
)

var (
	ErrVariantFieldNotFound = errors.New("variant field not found")
	ErrInvalidVariantField  = errors.New("variant field must be a string")
	ErrNoVariantFound       = errors.New("no variant found")
)

type variantEntry struct {
	Name  string
	Field reflect.Value
}

func UntaggedUnmarshalJSON(data []byte, v any) error {
	r := reflect.ValueOf(v)
	if r.Kind() == reflect.Ptr {
		r = r.Elem()
	} else {
		return errors.New("input must be a pointer to struct")
	}

	variantField, ok := searchVariantField(r)
	if !ok {
		return ErrVariantFieldNotFound
	}
	if variantField.Type().Kind() != reflect.String {
		return ErrInvalidVariantField
	}

	variants := getVariants(r)
	if len(variants) == 0 {
		return ErrNoVariantFound
	}

	var raw json.RawMessage
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}

	var lastError error
	for _, entry := range variants {
		err := json.Unmarshal(raw, entry.Field.Addr().Interface())
		if err == nil {
			variantField.SetString(entry.Name)
			return nil
		}
		lastError = err
	}

	return fmt.Errorf("all variants failed: %w", lastError)
}

func searchVariantField(v reflect.Value) (reflect.Value, bool) {
	for i := 0; i < v.NumField(); i++ {
		field := v.Field(i)
		tag, ok := v.Type().Field(i).Tag.Lookup("ujson")
		if !ok {
			continue
		}
		if tag == variantTag {
			return field, true
		}
	}
	return reflect.Value{}, false
}

func getVariants(v reflect.Value) []variantEntry {
	var variants []variantEntry
	for i := 0; i < v.NumField(); i++ {
		field := v.Field(i)
		tag, ok := v.Type().Field(i).Tag.Lookup("ujson")
		if !ok {
			continue
		}
		if tag == variantTag {
			continue
		}
		variants = append(variants, variantEntry{
			Name:  tag,
			Field: field,
		})
	}
	return variants
}

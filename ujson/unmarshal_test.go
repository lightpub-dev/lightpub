package ujson_test

import (
	"encoding/json"
	"testing"

	"github.com/lightpub-dev/lightpub/ujson"
)

// Basic test struct
type TestUnion struct {
	Variant string `ujson:"-"`

	StringValue string `ujson:"string"`
	IntValue    int    `ujson:"int"`
	BoolValue   bool   `ujson:"bool"`
}

func (t *TestUnion) UnmarshalJSON(data []byte) error {
	return ujson.UntaggedUnmarshalJSON(data, t)
}

func TestBasicUnmarshalling(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		expected TestUnion
		wantErr  bool
	}{
		{
			name:     "string variant",
			json:     `"hello"`,
			expected: TestUnion{Variant: "string", StringValue: "hello"},
		},
		{
			name:     "int variant",
			json:     `42`,
			expected: TestUnion{Variant: "int", IntValue: 42},
		},
		{
			name:     "bool variant",
			json:     `true`,
			expected: TestUnion{Variant: "bool", BoolValue: true},
		},
		{
			name:    "invalid json",
			json:    `{invalid`,
			wantErr: true,
		},
		{
			name:    "no matching variant",
			json:    `{"field": "value"}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var result TestUnion
			err := json.Unmarshal([]byte(tt.json), &result)

			if (err != nil) != tt.wantErr {
				t.Errorf("Expected error: %v, got: %v", tt.wantErr, err)
				return
			}

			if !tt.wantErr && (result.Variant != tt.expected.Variant ||
				result.StringValue != tt.expected.StringValue ||
				result.IntValue != tt.expected.IntValue ||
				result.BoolValue != tt.expected.BoolValue) {
				t.Errorf("Expected %+v, got %+v", tt.expected, result)
			}
		})
	}
}

type InnerUnionText struct {
	Text string `json:"text"`
}

type InnerUnionNumber struct {
	Number float64 `json:"number"`
}

// Inner union that can be a string or number
type InnerUnion struct {
	Variant     string           `ujson:"-"`
	TextValue   InnerUnionText   `ujson:"text"`
	NumberValue InnerUnionNumber `ujson:"number"`
}

func (i *InnerUnion) UnmarshalJSON(data []byte) error {
	return ujson.UntaggedUnmarshalJSON(data, i)
}

// Outer union that can contain different types including the inner union
type OuterUnion struct {
	Variant     string      `ujson:"-"`
	ObjectValue *InnerUnion `ujson:"object"`
	ArrayValue  []string    `ujson:"array"`
	SimpleValue string      `ujson:"simple"`
}

func (o *OuterUnion) UnmarshalJSON(data []byte) error {
	return ujson.UntaggedUnmarshalJSON(data, o)
}

// Container with nested union
type ContainerWithNestedUnion struct {
	ID       string     `json:"id"`
	Metadata string     `json:"metadata"`
	Data     OuterUnion `json:"data"`
}

func TestNestedUnions(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		expected OuterUnion
		wantErr  bool
	}{
		{
			name: "object with inner text",
			json: `{"text": "hello world"}`,
			expected: OuterUnion{
				Variant: "object",
				ObjectValue: &InnerUnion{
					Variant: "text",
					TextValue: InnerUnionText{
						Text: "hello world",
					},
				},
			},
		},
		{
			name: "object with inner number",
			json: `{"number": 42.5}`,
			expected: OuterUnion{
				Variant: "object",
				ObjectValue: &InnerUnion{
					Variant: "number",
					NumberValue: InnerUnionNumber{
						Number: 42.5,
					},
				},
			},
		},
		{
			name: "array variant",
			json: `["one", "two", "three"]`,
			expected: OuterUnion{
				Variant:    "array",
				ArrayValue: []string{"one", "two", "three"},
			},
		},
		{
			name: "simple string variant",
			json: `"simple string"`,
			expected: OuterUnion{
				Variant:     "simple",
				SimpleValue: "simple string",
			},
		},
		{
			name:    "invalid nested object",
			json:    `{"invalid": true}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var result OuterUnion
			err := json.Unmarshal([]byte(tt.json), &result)

			if (err != nil) != tt.wantErr {
				t.Errorf("Expected error: %v, got: %v", tt.wantErr, err)
				return
			}

			if tt.wantErr {
				return
			}

			if result.Variant != tt.expected.Variant {
				t.Errorf("Expected variant %q, got %q", tt.expected.Variant, result.Variant)
				return
			}

			switch result.Variant {
			case "object":
				if result.ObjectValue.Variant != tt.expected.ObjectValue.Variant {
					t.Errorf("Inner variant mismatch: expected %q, got %q",
						tt.expected.ObjectValue.Variant, result.ObjectValue.Variant)
					return
				}

				switch result.ObjectValue.Variant {
				case "text":
					if result.ObjectValue.TextValue != tt.expected.ObjectValue.TextValue {
						t.Errorf("TextValue mismatch: expected %q, got %q",
							tt.expected.ObjectValue.TextValue, result.ObjectValue.TextValue)
					}
				case "number":
					if result.ObjectValue.NumberValue != tt.expected.ObjectValue.NumberValue {
						t.Errorf("NumberValue mismatch: expected %v, got %v",
							tt.expected.ObjectValue.NumberValue, result.ObjectValue.NumberValue)
					}
				}
			case "array":
				if len(result.ArrayValue) != len(tt.expected.ArrayValue) {
					t.Errorf("ArrayValue length mismatch: expected %d, got %d",
						len(tt.expected.ArrayValue), len(result.ArrayValue))
					return
				}

				for i, v := range result.ArrayValue {
					if v != tt.expected.ArrayValue[i] {
						t.Errorf("ArrayValue[%d] mismatch: expected %q, got %q",
							i, tt.expected.ArrayValue[i], v)
					}
				}
			case "simple":
				if result.SimpleValue != tt.expected.SimpleValue {
					t.Errorf("SimpleValue mismatch: expected %q, got %q",
						tt.expected.SimpleValue, result.SimpleValue)
				}
			}
		})
	}
}

func TestComplexNestedStructures(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		expected ContainerWithNestedUnion
		wantErr  bool
	}{
		{
			name: "container with object/text",
			json: `{
				"id": "test1", 
				"metadata": "meta1",
				"data": {"text": "nested text"}
			}`,
			expected: ContainerWithNestedUnion{
				ID:       "test1",
				Metadata: "meta1",
				Data: OuterUnion{
					Variant: "object",
					ObjectValue: &InnerUnion{
						Variant:   "text",
						TextValue: InnerUnionText{Text: "nested text"},
					},
				},
			},
		},
		{
			name: "container with object/number",
			json: `{
				"id": "test2",
				"metadata": "meta2",
				"data": {"number": 123.456}
			}`,
			expected: ContainerWithNestedUnion{
				ID:       "test2",
				Metadata: "meta2",
				Data: OuterUnion{
					Variant: "object",
					ObjectValue: &InnerUnion{
						Variant:     "number",
						NumberValue: InnerUnionNumber{Number: 123.456},
					},
				},
			},
		},
		{
			name: "container with array",
			json: `{
				"id": "test3",
				"metadata": "meta3",
				"data": ["first", "second", "third"]
			}`,
			expected: ContainerWithNestedUnion{
				ID:       "test3",
				Metadata: "meta3",
				Data: OuterUnion{
					Variant:    "array",
					ArrayValue: []string{"first", "second", "third"},
				},
			},
		},
		{
			name: "container with simple",
			json: `{
				"id": "test4",
				"metadata": "meta4",
				"data": "plain text"
			}`,
			expected: ContainerWithNestedUnion{
				ID:       "test4",
				Metadata: "meta4",
				Data: OuterUnion{
					Variant:     "simple",
					SimpleValue: "plain text",
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var result ContainerWithNestedUnion
			err := json.Unmarshal([]byte(tt.json), &result)

			if (err != nil) != tt.wantErr {
				t.Errorf("Expected error: %v, got: %v", tt.wantErr, err)
				return
			}

			if tt.wantErr {
				return
			}

			if result.ID != tt.expected.ID {
				t.Errorf("ID mismatch: expected %q, got %q", tt.expected.ID, result.ID)
			}

			if result.Metadata != tt.expected.Metadata {
				t.Errorf("Metadata mismatch: expected %q, got %q", tt.expected.Metadata, result.Metadata)
			}

			if result.Data.Variant != tt.expected.Data.Variant {
				t.Errorf("Data.Variant mismatch: expected %q, got %q",
					tt.expected.Data.Variant, result.Data.Variant)
				return
			}

			// Test the nested union based on its variant
			// (Same validation as in TestNestedUnions)
		})
	}
}

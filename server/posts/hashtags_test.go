package posts_test

import (
	"reflect"
	"testing"

	"github.com/lightpub-dev/lightpub/posts"
)

func TestFindHashtags(t *testing.T) {
	tests := []struct {
		name     string
		content  string
		expected []string
	}{
		{
			name:     "Single hashtag",
			content:  "Here is a post with a #hashtag",
			expected: []string{"#hashtag"},
		},
		{
			name:     "Multiple hashtags",
			content:  "This post contains multiple #hashtags, with #different #tags.",
			expected: []string{"#hashtags", "#different", "#tags"},
		},
		{
			name:     "No hashtags",
			content:  "This is a post without any hashtags.",
			expected: []string{},
		},
		{
			name:     "Hashtags with numbers",
			content:  "Hashtags can have numbers like #tag1 and #2tag",
			expected: []string{"#tag1", "#2tag"},
		},
		{
			name:     "Non-latin characters",
			content:  "Hashtags with non-Latin characters #тег #标签",
			expected: []string{"#тег", "#标签"},
		},
		{
			name:     "Repeated hashtags",
			content:  "Some posts repeat hashtags #tag #other #tag",
			expected: []string{"#tag", "#other"},
		},
		{
			name:     "Hashtag at the end",
			content:  "Hashtag at the end #end",
			expected: []string{"#end"},
		},
		{
			name:     "Hashtag with punctuation",
			content:  "Hashtags followed by punctuation #tag!",
			expected: []string{"#tag"},
		},
		{
			name:     "Empty string",
			content:  "",
			expected: []string{},
		},
		{
			name:     "String with just #",
			content:  "#",
			expected: []string{"#"},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if got := posts.FindHashtags(test.content); !reflect.DeepEqual(got, test.expected) {
				t.Errorf("FindHashtags() = %v, want %v", got, test.expected)
			}
		})
	}
}

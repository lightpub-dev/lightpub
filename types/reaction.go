package types

import (
	"unicode/utf8"

	"github.com/lightpub-dev/lightpub/failure"
	"github.com/rivo/uniseg"
)

var (
	ErrNotEmoji = failure.NewError(400, "not an emoji")
)

type NoteReactionContent interface {
	ReactionAsText() string
}

type EmojiNoteReaction struct {
	Emoji string
}

func (e EmojiNoteReaction) ReactionAsText() string {
	return e.Emoji
}

func NewEmojiNoteReaction(emoji string) (EmojiNoteReaction, error) {
	if !IsOneEmoji(emoji) {
		return EmojiNoteReaction{}, ErrNotEmoji
	}
	return EmojiNoteReaction{Emoji: emoji}, nil
}

func IsOneEmoji(s string) bool {
	// Check if the string is empty
	if s == "" {
		return false
	}

	// Ensure it's exactly one grapheme cluster
	graphemes := uniseg.NewGraphemes(s)
	graphemes.Next()
	if graphemes.Next() {
		// More than one grapheme
		return false
	}

	// Check if it's an emoji
	// Option 1: Check the unicode range (simplified approach)
	r, _ := utf8.DecodeRuneInString(s)
	if (r >= 0x1F300 && r <= 0x1F5FF) || // Miscellaneous Symbols and Pictographs
		(r >= 0x1F600 && r <= 0x1F64F) || // Emoticons
		(r >= 0x1F680 && r <= 0x1F6FF) || // Transport and Map Symbols
		(r >= 0x1F700 && r <= 0x1F77F) || // Alchemical Symbols
		(r >= 0x1F780 && r <= 0x1F7FF) || // Geometric Shapes Extended
		(r >= 0x1F800 && r <= 0x1F8FF) || // Supplemental Arrows-C
		(r >= 0x1F900 && r <= 0x1F9FF) || // Supplemental Symbols and Pictographs
		(r >= 0x2600 && r <= 0x26FF) || // Miscellaneous Symbols
		(r >= 0x2700 && r <= 0x27BF) { // Dingbats
		return true
	}

	return false
}

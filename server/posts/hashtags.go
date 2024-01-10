package posts

import "unicode"

func FindHashtags(content string) []string {
	hashtags := []string{}
	inHashtag := -1
	ignoreCurrentWord := false
	for i, ch := range content {
		if ch == '#' {
			if inHashtag >= 0 {
				inHashtag = -1
				ignoreCurrentWord = true
			} else {
				inHashtag = i
			}
			continue
		}

		isWordBoundary := !unicode.IsLetter(ch) && !unicode.IsNumber(ch) && ch != '_' && ch != '-'
		if isWordBoundary {
			if inHashtag >= 0 {
				hashtagStr := content[inHashtag:i]
				hashtags = append(hashtags, hashtagStr)

				inHashtag = -1
				ignoreCurrentWord = false
			}
			continue
		}
	}

	if inHashtag >= 0 && !ignoreCurrentWord {
		hashtagStr := content[inHashtag:]
		hashtags = append(hashtags, hashtagStr)
	}

	// remove duplicates
	uniqueHashtags := []string{}
	seen := map[string]bool{}
	for _, hashtag := range hashtags {
		if !seen[hashtag] {
			uniqueHashtags = append(uniqueHashtags, hashtag)
			seen[hashtag] = true
		}
	}

	return uniqueHashtags
}

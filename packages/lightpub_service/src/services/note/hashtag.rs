/*
Lightpub: a simple ActivityPub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#[derive(Debug, PartialEq)]
pub struct Hashtag {
    pub(crate) hashtag: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub fn find_hashtags(content: &str) -> Vec<Hashtag> {
    let mut hashtags: Vec<Hashtag> = Vec::new();

    // Split content by spaces (including Japanese spaces)
    let words = content.split_whitespace();

    for word in words {
        let trimmed_word = word.trim();
        if !trimmed_word.starts_with("#") {
            continue;
        }
        if trimmed_word.len() < 2 {
            continue;
        }

        // Use find() to get the first occurrence's index.
        if let Some(start) = content.find(trimmed_word) {
            hashtags.push(Hashtag {
                hashtag: trimmed_word.to_string(),
                start,
                end: start + trimmed_word.len(),
            });
        }
    }

    hashtags
}

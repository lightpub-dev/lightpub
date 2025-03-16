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

use derive_getters::Getters;
use regex::Regex;

use crate::services::user::UserSpecifier;

#[derive(Debug, PartialEq, Getters)]
pub struct Mention {
    username: String,
    domain: Option<String>,
    start: usize,
    end: usize,
}

pub fn find_mentions(content: &str, my_domain: &str) -> Vec<Mention> {
    let re = Regex::new(r"\B(@[a-zA-Z0-9_-]+(?:@[.a-zA-Z_\-0-9]+)?)").unwrap();
    let mut mentions: Vec<Mention> = Vec::new();

    for cap in re.captures_iter(content) {
        if let Some(matched_str) = cap.get(1) {
            if let Some(spec) = UserSpecifier::Specifier(matched_str.as_str().to_owned())
                .try_parse_specifier(my_domain)
            {
                match spec {
                    UserSpecifier::Username(username, domain) => {
                        mentions.push(Mention {
                            username,
                            domain,
                            start: matched_str.start(),
                            end: matched_str.end(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    mentions
}

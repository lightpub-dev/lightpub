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

use pulldown_cmark::{Parser, html};

use async_trait::async_trait;

use crate::utils::sanitize::CleanString;

#[async_trait]
pub trait NoteRenderer {
    async fn render_note(&self, content: &str) -> CleanString;
}

pub struct PlainNoteRenderer {}

impl PlainNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NoteRenderer for PlainNoteRenderer {
    async fn render_note(&self, content: &str) -> CleanString {
        CleanString::clean_text(content)
    }
}

pub struct MdNoteRenderer {}

impl MdNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NoteRenderer for MdNoteRenderer {
    async fn render_note(&self, content: &str) -> CleanString {
        let parser = Parser::new(content);

        let mut html_buf = String::new();
        html::push_html(&mut html_buf, parser);

        CleanString::clean(&html_buf)
    }
}

pub struct HtmlNoteRenderer {}

impl HtmlNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NoteRenderer for HtmlNoteRenderer {
    async fn render_note(&self, content: &str) -> CleanString {
        CleanString::clean(content)
    }
}

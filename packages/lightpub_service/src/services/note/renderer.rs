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

use derive_more::Constructor;
use pulldown_cmark::{Parser, html};

use serde::Deserialize;
use serde_json::json;

use crate::{ServiceResult, services::queue::QConn, utils::sanitize::CleanString};

const MATHJAX_RENDERER_SUBJECT: &str = "lightpub.mathjax.render";

pub struct PlainNoteRenderer {}

impl PlainNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn render_note(&self, content: &str) -> ServiceResult<CleanString> {
        Ok(CleanString::clean_text(content))
    }
}

pub struct MdNoteRenderer {}

impl MdNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn render_note(&self, content: &str) -> ServiceResult<CleanString> {
        let parser = Parser::new(content);

        let mut html_buf = String::new();
        html::push_html(&mut html_buf, parser);

        Ok(CleanString::clean(&html_buf))
    }
}

pub struct HtmlNoteRenderer {}

impl HtmlNoteRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn render_note(&self, content: &str) -> ServiceResult<CleanString> {
        Ok(CleanString::clean(content))
    }
}

#[derive(Debug, Constructor)]
pub struct LatexNoteRenderer {}

impl LatexNoteRenderer {
    pub async fn render_note(&self, content: &str, qconn: &QConn) -> ServiceResult<CleanString> {
        let content: MathJaxResponse = qconn
            .request(
                MATHJAX_RENDERER_SUBJECT,
                &json!({
                    "content": content
                }),
            )
            .await?;
        Ok(CleanString::already_cleaned_dangerous(content.result))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MathJaxResponse {
    result: String,
}

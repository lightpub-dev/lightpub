use derive_more::Constructor;

#[derive(Constructor)]
pub struct PostContentService {}

impl PostContentService {
    pub fn html_to_internal(&self, html: &str) -> String {
        // strip all html tags
        let frag = scraper::Html::parse_fragment(html);
        let mut text = String::new();
        for node in frag.tree {
            if let scraper::Node::Text(text_node) = node {
                text.push_str(&text_node);
            }
        }
        text
    }
}

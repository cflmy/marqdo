//! Markdown → HTML for article bodies (GFM subset via pulldown-cmark).

use pulldown_cmark::{html, Options, Parser};

pub fn to_html(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS,
    );
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    format!("<div class=\"article-body md\">{html_out}</div>")
}

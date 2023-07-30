#[cfg(test)]

use crate::html_lexer::HtmlToken;


pub fn html_doctype(text: &str) -> HtmlToken { return HtmlToken::Doctype(text.to_owned()); }
pub fn html_text(text: &str) -> HtmlToken { return HtmlToken::Text(text.to_owned()); }
pub fn html_whitespace(text: &str) -> HtmlToken { return HtmlToken::Whitespace(text.to_owned()); }
pub fn html_open(tag_name: &str) -> HtmlToken { return HtmlToken::OpenTag{ name: tag_name.to_owned() }; }
pub fn html_close(tag_name: &str) -> HtmlToken { return HtmlToken::CloseTag{ name: tag_name.to_owned() }; }
pub fn html_open_tag_end() -> HtmlToken { return HtmlToken::OpenTagEnd; }
pub fn html_comment(text: &str) -> HtmlToken { return HtmlToken::Comment(text.to_owned()); }

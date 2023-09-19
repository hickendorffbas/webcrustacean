use std::sync::atomic::{AtomicUsize, Ordering};

use crate::html_lexer::{AttributeContent, HtmlToken, HtmlTokenWithLocation};


static NEXT_TEST_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_test_id() -> usize { NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed) }


pub fn tokens_equal_ignoring_location(actual_tokens: Vec<HtmlTokenWithLocation>, expected_tokens: Vec<HtmlTokenWithLocation>) -> bool {
    for (actual_token, expected_token) in actual_tokens.iter().zip(expected_tokens.iter()) {
        if actual_token.html_token != expected_token.html_token {
            return false;
        }
    }
    return true;
}


pub fn html_doctype_loc(text: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::Doctype(text.to_owned()), line: line_nr, character: character_nr };
}
pub fn html_doctype(text: &str) -> HtmlTokenWithLocation { return html_doctype_loc(text, 0, 0) }


pub fn html_text_loc(text: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::Text(text.to_owned()), line: line_nr, character: character_nr };
}
pub fn html_text(text: &str) -> HtmlTokenWithLocation { return html_text_loc(text, 0, 0) }


pub fn html_whitespace_loc(text: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::Whitespace(text.to_owned()), line: line_nr, character: character_nr };
}
pub fn html_whitespace(text: &str) -> HtmlTokenWithLocation { return html_whitespace_loc(text, 0, 0); }


pub fn html_open_loc(tag_name: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::OpenTag{ name: tag_name.to_owned() }, line: line_nr, character: character_nr };
}
pub fn html_open(text: &str) -> HtmlTokenWithLocation { return html_open_loc(text, 0, 0); }


pub fn html_close_loc(tag_name: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::CloseTag{ name: tag_name.to_owned() }, line: line_nr, character: character_nr };
}
pub fn html_close(text: &str) -> HtmlTokenWithLocation { return html_close_loc(text, 0, 0); }


pub fn html_open_tag_end_loc(line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::OpenTagEnd, line: line_nr, character: character_nr };
}
pub fn html_open_tag_end() -> HtmlTokenWithLocation { return html_open_tag_end_loc(0, 0); }


pub fn html_comment_loc(text: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::Comment(text.to_owned()), line: line_nr, character: character_nr };
}
pub fn html_comment(text: &str) -> HtmlTokenWithLocation { return html_comment_loc(text, 0, 0); }


pub fn html_attribute_loc(name: &str, value: &str, line_nr: u32, character_nr: u32) -> HtmlTokenWithLocation {
    return HtmlTokenWithLocation { html_token: HtmlToken::Attribute(AttributeContent { name: name.to_owned(), value: value.to_owned() }),
                                   line: line_nr, character: character_nr };
}
pub fn html_attribute(name: &str, value: &str) -> HtmlTokenWithLocation { return html_attribute_loc(name, value, 0, 0); }


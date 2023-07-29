use crate::html_lexer::{self, HtmlToken};


#[test]
fn test_basic_tokenisation_1() {
    let html = "<html>test\n   <b>bold</b> </html>";

    let expected_tokens = vec![
        t_open("html"),
        t_open_tag_end(),

        t_text("test"),
        t_whitespace("\n   "),

        t_open("b"),
        t_open_tag_end(),
        t_text("bold"),
        t_close("b"),

        t_whitespace(" "),

        t_close("html"),
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}


#[test]
fn test_basic_tokenisation_self_closing_tag() {
    let html = "text<br /> text";

    let expected_tokens = vec![
        t_text("text"),

        t_open("br"),
        t_open_tag_end(),
        t_close("br"),

        t_whitespace(" "),
        t_text("text"),
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}


fn t_text(text: &str) -> HtmlToken { return HtmlToken::Text(text.to_owned()); }
fn t_whitespace(text: &str) -> HtmlToken { return HtmlToken::Whitespace(text.to_owned()); }
fn t_open(tag_name: &str) -> HtmlToken { return HtmlToken::OpenTag{ name: tag_name.to_owned() }; }
fn t_close(tag_name: &str) -> HtmlToken { return HtmlToken::CloseTag{ name: tag_name.to_owned() }; }
fn t_open_tag_end() -> HtmlToken { return HtmlToken::OpenTagEnd; }

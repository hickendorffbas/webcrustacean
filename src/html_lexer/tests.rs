use crate::html_lexer;
use crate::test_util::*;


#[test]
fn test_basic_tokenisation_1() {
    let html = "<html>test\n   <b>bold</b> </html>";

    let expected_tokens = vec![
        html_open("html"),
        html_open_tag_end(),

        html_text("test"),
        html_whitespace("\n   "),

        html_open("b"),
        html_open_tag_end(),
        html_text("bold"),
        html_close("b"),

        html_whitespace(" "),

        html_close("html"),
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}


#[test]
fn test_self_closing_tag() {
    let html = "text<br /> text";

    let expected_tokens = vec![
        html_text("text"),

        html_open("br"),
        html_open_tag_end(),
        html_close("br"),

        html_whitespace(" "),
        html_text("text"),
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}


#[test]
fn test_doctype() {
    let html = "<!DOCTYPE html>
    <html>
    </html>";

    let expected_tokens = vec![
        html_doctype(" html"), //TODO: would be good to not have the leading space here (strip the string or something?)
        html_whitespace("\n    "),

        html_open("html"),
        html_open_tag_end(),
        html_whitespace("\n    "),

        html_close("html"),
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}


#[test]
fn test_comment() {
    let html = "<x>a</x><!-- this is a comment -->";

    let expected_tokens = vec![
        html_open("x"),
        html_open_tag_end(),
        html_text("a"),
        html_close("x"),
        html_comment(" this is a comment --")
    ];

    let tokens = html_lexer::lex_html(html);
    assert_eq!(tokens, expected_tokens);
}
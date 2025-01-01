use std::iter::Peekable;
use std::str::Chars;

use crate::debug::debug_log_warn;

#[cfg(test)] mod tests;


const DOCTYPE_CHARS: [char; 7] = ['d', 'o', 'c', 't', 'y', 'p', 'e'];


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct HtmlTokenWithLocation {
    pub html_token: HtmlToken,
    pub line: u32,
    pub character: u32
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum HtmlToken {
    OpenTag{name: String},
    OpenTagEnd,
    Attribute(AttributeContent),
    CloseTag{name: String},
    Text(String),
    Whitespace(String),
    Comment(String),
    Doctype(String),
    Entity(String),
    Style(String),
    Script(String),
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct AttributeContent {
    pub name: String,
    pub value: String,
}


//TODO: move this iterator to a parsing.rs or something, since it is also used for javascript
pub struct TrackingIterator<'document> {
    pub iter: Peekable<Chars<'document>>,
    pub current_line: u32,
    pub current_char: u32,
}
impl TrackingIterator<'_> {
    pub fn next(&mut self) -> char {
        let next_char = self.iter.next().unwrap();
        if next_char == '\n' {
            self.current_line += 1;
            self.current_char = 1;
        } else {
            self.current_char += 1;
        }
        return next_char;
    }

    pub fn peek(&mut self) -> Option<&char> {
        return self.iter.peek();
    }

    pub fn has_next(&mut self) -> bool {
        return self.iter.peek().is_some();
    }

    pub fn next_tokens_no_whitespace_are(&mut self, text_to_check: &str) -> bool {
        let mut temp_html_iterator = self.iter.clone();
        let mut expected_iterator = text_to_check.chars();

        loop {
            let mut next_expected = expected_iterator.next();
            while next_expected.is_some() && next_expected.unwrap() == ' ' {
                next_expected = expected_iterator.next();
            }
            if next_expected.is_none() {
                return true;
            }

            let mut next_in_html = temp_html_iterator.next();
            while next_in_html.is_some() && next_in_html.unwrap() == ' ' {
                next_in_html = temp_html_iterator.next();
            }
            if next_in_html.is_none() {
                return false;
            }

            if next_in_html.unwrap() != next_expected.unwrap() {
                return false;
            }
        }
    }
}


pub fn lex_html(document: &str) -> Vec<HtmlTokenWithLocation> {
    let mut tokens: Vec<HtmlTokenWithLocation> = Vec::new();

    let mut html_iterator = TrackingIterator {
        iter: document.chars().peekable(),
        current_line: 1,
        current_char: 0,
    };


    while html_iterator.has_next() {
        let next_char = html_iterator.next();
        let line_nr = html_iterator.current_line;
        let char_nr = html_iterator.current_char;

        match next_char {
            '<' => {
                eat_whitespace(&mut html_iterator);

                if let Some('/') = html_iterator.peek() {  //we are reading a closing tag
                    html_iterator.next();
                    eat_whitespace(&mut html_iterator);

                    let tag_name = consume_full_name(&mut html_iterator);
                    eat_whitespace(&mut html_iterator);

                    if let Some('>') = html_iterator.peek() {
                        html_iterator.next();
                    } else {
                        //TODO: we should probably handle extra stuff after the tagname differently (check what actual browsers do)
                        todo!("This case is not valid html, but we should still handle it in some way");
                    }

                    tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::CloseTag {name: tag_name}, line: line_nr, character: char_nr } );

                } else if let Some('!') = html_iterator.peek() {  //we are reading a comment or doctype
                    html_iterator.next(); //eat the !

                    if let Some('-') = html_iterator.peek() {
                        html_iterator.next(); //eat the -
                        if let Some('-') = html_iterator.peek() { //we are reading a comment
                            html_iterator.next(); //eat the -

                            let comment_content = lex_comment(&mut html_iterator);
                            tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Comment(comment_content), line: line_nr, character: char_nr } );
                        } else {
                            debug_log_warn(format!("Unexpected chars after <! ({}:{})", line_nr, char_nr));
                        }
                    } else {
                        let mut is_doctype = true;

                        for current_char in DOCTYPE_CHARS {
                            if html_iterator.has_next() {
                                if (*html_iterator.peek().unwrap()).to_ascii_lowercase() == current_char {
                                    html_iterator.next(); //TODO: ideally we don't consume from the iterator until we are sure all the chars match
                                } else {
                                    is_doctype = false;
                                    break;
                                }
                            } else {
                                is_doctype = false;
                                break;
                            }
                        }

                        if is_doctype {
                            let rest_of_tag_content = consume_until_char(&mut html_iterator, '>');
                            tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Doctype(rest_of_tag_content), line: line_nr, character: char_nr } );
                        } else {
                            todo!(); //TODO: implement
                        }

                    }

                } else { //we are reading an opening tag
                    eat_whitespace(&mut html_iterator);

                    let tag_name = consume_full_name(&mut html_iterator);
                    eat_whitespace(&mut html_iterator);

                    tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::OpenTag {name: tag_name.clone()}, line: line_nr, character: char_nr } );

                    while html_iterator.has_next() &&
                          html_iterator.peek().unwrap() != &'>' && html_iterator.peek().unwrap() != &'/' {
                        let att_line = html_iterator.current_line;
                        let att_char = html_iterator.current_char;
                        tokens.push(HtmlTokenWithLocation { html_token: consume_tag_attribute(&mut html_iterator), line: att_line, character: att_char } );
                        eat_whitespace(&mut html_iterator);
                    }

                    let next_char = html_iterator.peek().unwrap();
                    if next_char == &'/' {
                        // We are in a self-closing tag
                        html_iterator.next(); //read the '/'

                        eat_whitespace(&mut html_iterator);

                        html_iterator.next(); //read the '>'

                        tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::OpenTagEnd,
                                                            line: html_iterator.current_line,
                                                            character: html_iterator.current_char } );
                        tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::CloseTag { name: tag_name },
                                                            line: html_iterator.current_line,
                                                            character: html_iterator.current_char } );

                    } else if next_char == &'>' {
                        html_iterator.next(); //read the '>'
                        tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::OpenTagEnd,
                                                            line: html_iterator.current_line,
                                                            character: html_iterator.current_char } );

                        if tag_name == "style" { //TODO: do we already lower() the tag names?

                            let mut css_data = String::new();
                            while html_iterator.has_next() && !html_iterator.next_tokens_no_whitespace_are("</style>") {
                                css_data.push(html_iterator.next());
                            }

                            tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Style(css_data),
                                                                line: html_iterator.current_line, //TODO: line (and char below are wrong, should be start positions...)
                                                                character: html_iterator.current_char });
                        }

                        if tag_name == "script" { //TODO: do we already lower() the tag names?

                            let mut in_quotes = false;
                            let mut script_data = String::new();
                            while html_iterator.has_next() && !(!in_quotes && html_iterator.next_tokens_no_whitespace_are("</script>")) {

                                let next_char = html_iterator.next();
                                if next_char == '"' {
                                    in_quotes = !in_quotes;
                                }
                                script_data.push(next_char);
                            }

                            tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Script(script_data),
                                                                line: html_iterator.current_line, //TODO: line (and char below are wrong, should be start positions...)
                                                                character: html_iterator.current_char });
                        }


                    } else {
                        //Given the while loop above, this should not be reachable
                        panic!("Illegal state");
                    }

                }
            },
            '&' => {
                let entity_data = consume_until_char(&mut html_iterator, ';');
                tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Entity(entity_data), line: line_nr, character: char_nr } );
            },
            ' ' | '\n' | '\t' | '\r' => {
                let mut str_buffer = String::new();
                str_buffer.push(next_char);

                while html_iterator.has_next() && is_whitespace(*html_iterator.peek().unwrap()) {
                    str_buffer.push(html_iterator.next());
                }

                tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Whitespace(str_buffer), line: line_nr, character: char_nr } );
            }
            _ => {
                let mut str_buffer = next_char.to_string();

                while html_iterator.has_next() {
                    let c = *html_iterator.peek().unwrap();

                    if is_whitespace(c) || c == '<' || c == '&' {
                        break;
                    }
                    str_buffer.push(html_iterator.next());
                }
                tokens.push(HtmlTokenWithLocation { html_token: HtmlToken::Text(str_buffer), line: line_nr, character: char_nr } );
            },
        }

    }

    return tokens;
}


fn consume_tag_attribute(html_iterator: &mut TrackingIterator) -> HtmlToken {
    let attribute_name = consume_full_name(html_iterator);

    let mut attribute_value: String;
    eat_whitespace(html_iterator);
    if let Some('=') = html_iterator.peek() {
        html_iterator.next();
        eat_whitespace(html_iterator);

        if let Some('"') = html_iterator.peek() {
            html_iterator.next(); //eat the quote

            attribute_value = String::new();
            while html_iterator.has_next() && html_iterator.peek().unwrap() != &'"' {
                attribute_value.push(html_iterator.next());
            }
            html_iterator.next(); //eat the quote

        } else if let Some('\'') = html_iterator.peek() {
            html_iterator.next(); //eat the quote

            attribute_value = String::new();
            while html_iterator.has_next() && html_iterator.peek().unwrap() != &'\'' {
                attribute_value.push(html_iterator.next());
            }
            html_iterator.next(); //eat the quote

        } else {
            //no quotes in the attributes value, so we read until next whitespace or other special char
            attribute_value = consume_full_name(html_iterator);
        }
    } else {
        //this is the case where the attribute does not have "="
        attribute_value = attribute_name.clone();
    }

    return HtmlToken::Attribute(AttributeContent{name: attribute_name, value: attribute_value});
}


fn consume_until_char(html_iterator: &mut TrackingIterator, limit: char) -> String {
    let mut str_buffer = String::new();
    while html_iterator.has_next() && *html_iterator.peek().unwrap() != limit {
        str_buffer.push(html_iterator.next());
    }
    html_iterator.next(); //eat the limit char
    return str_buffer;
}


fn lex_comment(html_iterator: &mut TrackingIterator) -> String {
    let mut buffer = String::new();

    while html_iterator.has_next() {
        let cur_char = html_iterator.next();

        if cur_char == '-' {
            let cur_char = html_iterator.next();
            if cur_char == '-' {
                let cur_char = html_iterator.next();
                if cur_char == '>' {
                    return buffer;
                } else {
                    buffer.push('-');
                    buffer.push('-');
                    buffer.push(cur_char);
                }
            } else {
                buffer.push('-');
                buffer.push(cur_char);
            }
        } else {
            buffer.push(cur_char);
        }
    }

    return buffer;
}


fn consume_full_name(iterator: &mut TrackingIterator) -> String {
    let mut str_buffer = String::new();
    loop {
        let opt_peek = iterator.peek();
        if opt_peek.is_none() {
            return str_buffer
        }

        let peek = *opt_peek.unwrap();
        if !is_whitespace(peek) && peek != '=' && peek != '<' && peek != '>'  {
            str_buffer.push(iterator.next());
        } else {
            return str_buffer
        }
    }
}


fn eat_whitespace(iterator: &mut TrackingIterator) {
    loop {
        let opt_peek = iterator.peek();
        if opt_peek.is_none() {
            return
        }

        let peek = *opt_peek.unwrap();

        if is_whitespace(peek) {
            iterator.next();
        } else {
            return
        }
    }
}

fn is_whitespace(c: char) -> bool {
    return c == ' ' || c == '\n' || c == '\t' || c == '\r';
}
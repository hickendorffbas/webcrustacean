use std::iter::Peekable;
use std::str::Chars;

use crate::debug::debug_print_html_tokens;

#[cfg(test)]
mod tests;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum HtmlToken {
    OpenTag{name: String},
    OpenTagEnd,
    Attribute(AttributeContent),
    CloseTag{name: String},
    Text(String),
    Whitespace(String),
    #[allow(dead_code)] //TODO: implement
    Comment(String),
    #[allow(dead_code)] //TODO: implement
    Doctype(String),
    #[allow(dead_code)] //TODO: implement
    EmpData(String), //Any &... entity
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct AttributeContent {
    pub name: String,
    pub value: String,
}


pub fn lex_html(document: &str) -> Vec<HtmlToken> {
    let mut tokens = Vec::new();
    let mut doc_iterator = document.chars().peekable();

    while doc_iterator.peek().is_some() {
        let next_char = doc_iterator.next().unwrap();

        match next_char {
            '<' => {
                eat_whitespace(&mut doc_iterator);

                if let Some('/') = doc_iterator.peek() {  //we are reading a closing tag
                    doc_iterator.next();
                    eat_whitespace(&mut doc_iterator);

                    let tag_name = consume_full_name(&mut doc_iterator);
                    eat_whitespace(&mut doc_iterator);

                    if let Some('>') = doc_iterator.peek() {
                        doc_iterator.next();
                    } else {
                        //TODO: we should probably handle extra stuff after the tagname differently (check what actual browsers do)
                        panic!("This case is not valid html, but we should still handle it in some way");
                    }

                    tokens.push(HtmlToken::CloseTag {name: tag_name} );

                } else { //we are reading an opening tag
                    eat_whitespace(&mut doc_iterator);

                    let tag_name = consume_full_name(&mut doc_iterator);
                    eat_whitespace(&mut doc_iterator);

                    tokens.push(HtmlToken::OpenTag {name: tag_name.clone()} );

                    while doc_iterator.peek().is_some() &&
                          doc_iterator.peek().unwrap() != &'>' && doc_iterator.peek().unwrap() != &'/' {
                        println!("processing {:?}", doc_iterator.peek());
                        tokens.push(consume_tag_attribute(&mut doc_iterator));
                        eat_whitespace(&mut doc_iterator);
                    }

                    let next_char = doc_iterator.peek().unwrap();
                    if next_char == &'/' {
                        // We are in a self-closing tag
                        doc_iterator.next(); //read the '/'

                        eat_whitespace(&mut doc_iterator);

                        doc_iterator.next(); //read the '>'
                        tokens.push(HtmlToken::OpenTagEnd);
                        tokens.push(HtmlToken::CloseTag { name: tag_name });

                    } else if next_char == &'>' {
                        doc_iterator.next(); //read the '>'
                        tokens.push(HtmlToken::OpenTagEnd);
                    } else {
                        //Given the while loop above, this should not be reachable
                        panic!("Illegal state");
                    }

                }
            },
            '&' => {
                //TODO: implement
                panic!("implement");
            },
            ' ' | '\n' | '\t' | '\r' => {
                let mut str_buffer = String::new();
                str_buffer.push(next_char);

                while doc_iterator.peek().is_some() && is_whitespace(*doc_iterator.peek().unwrap()) {
                    let whitespace_char = doc_iterator.next().unwrap();
                    str_buffer.push(whitespace_char);
                }

                tokens.push(HtmlToken::Whitespace(str_buffer));
            }
            _ => {
                let mut str_buffer = next_char.to_string();

                while doc_iterator.peek().is_some() {
                    let c = *doc_iterator.peek().unwrap();

                    if is_whitespace(c) || c == '<' || c == '&' {
                        break;
                    }
                    str_buffer.push(doc_iterator.next().unwrap());
                }
                tokens.push(HtmlToken::Text(str_buffer));
            },
        }

    }

    debug_print_html_tokens(&tokens);
    return tokens;
}


fn consume_tag_attribute(doc_iterator: &mut Peekable<Chars<'_>>) -> HtmlToken {
    let attribute_name = consume_full_name(doc_iterator);

    let mut attribute_value: String;
    eat_whitespace(doc_iterator);
    if let Some('=') = doc_iterator.peek() {
        doc_iterator.next();
        eat_whitespace(doc_iterator);

        if let Some('"') = doc_iterator.peek() {
            doc_iterator.next(); //eat the quote

            attribute_value = String::new();
            while doc_iterator.peek().is_some() && doc_iterator.peek().unwrap() != &'"' {
                attribute_value.push(doc_iterator.next().unwrap());
            }
            doc_iterator.next(); //eat the quote

        } else {
            //no quotes in the attributes value, so we read until next whitespace or other special char
            attribute_value = consume_full_name(doc_iterator);
        }
    } else {
        //this is the case where the attribute does not have "="
        attribute_value = attribute_name.clone();
    }

    return HtmlToken::Attribute(AttributeContent{name: attribute_name, value: attribute_value});
}


fn consume_full_name(iterator: &mut Peekable<Chars<'_>>) -> String {
    let mut str_buffer = String::new();
    loop {
        let opt_peek = iterator.peek();
        if opt_peek.is_none() {
            return str_buffer
        }

        let peek = *opt_peek.unwrap();

        if (peek >= 'a' && peek <= 'z') || (peek >= 'A' && peek <= 'Z') || (peek >= '0' && peek <= '9') {
            str_buffer.push(iterator.next().unwrap());
        } else {
            return str_buffer
        }
    }
}


fn eat_whitespace(iterator: &mut Peekable<Chars<'_>>) {
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
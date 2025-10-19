use crate::html_lexer::TrackingIterator;


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct CssTokenWithLocation {
    pub css_token: CssToken,
    pub line: u32,
    pub character: u32
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum CssToken {
    Selector(String),
    Property(String),
    Value(String),
    BlockStart,
    BlockEnd,
    Comma,
    #[allow(dead_code)] DescendentCombinator,  //TODO: lex this one
    #[allow(dead_code)] ChildCombinator,  //TODO: lex this one
    #[allow(dead_code)] SubsequentSiblingCombinator,  //TODO: lex this one
    #[allow(dead_code)] NextSiblingCombinator,  //TODO: lex this one
}


pub fn lex_css(document: &str, starting_line: u32, starting_char_idx: u32) -> Vec<CssTokenWithLocation> {
    let mut tokens = Vec::new();

    let mut css_iterator = TrackingIterator {
        iter: document.chars().peekable(),
        current_line: starting_line,
        current_char: starting_char_idx,
    };

    while css_iterator.has_next() {
        lex_css_block(&mut css_iterator, &mut tokens);
    }

    return tokens;
}


fn make_token(css_iterator: &mut TrackingIterator, token: CssToken) -> CssTokenWithLocation {
    return CssTokenWithLocation { css_token: token, line: css_iterator.current_line, character: css_iterator.current_char };
}


fn lex_css_block(css_iterator: &mut TrackingIterator, tokens: &mut Vec<CssTokenWithLocation>) {
    let mut buffer = String::new();
    let mut in_comment = false;

    'main_loop: while css_iterator.has_next() {

        if in_comment {
            match css_iterator.next() {
                '*' => {
                    if css_iterator.peek() == Some(&'/') {
                        css_iterator.next(); //eat the /
                        in_comment = false;
                    }
                },
                _ => {}
            }
            continue 'main_loop
        }

        match css_iterator.next() {
            '/' => {
                if css_iterator.peek() == Some(&'*') {
                    css_iterator.next(); //eat the *
                    in_comment = true;
                } else {
                    buffer.push('/');
                }
            },
            '{' => {
                if buffer.trim().len() > 0 {
                    tokens.push(make_token(css_iterator, CssToken::Selector(buffer.trim().to_owned())));
                    buffer.clear();
                }
                tokens.push(make_token(css_iterator, CssToken::BlockStart));
            },
            ',' => {
                tokens.push(make_token(css_iterator, CssToken::Selector(buffer.trim().to_owned())));
                buffer.clear();

                tokens.push(make_token(css_iterator, CssToken::Comma));
            },
            '}' => {
                if buffer.trim().len() > 0 {
                    tokens.push(make_token(css_iterator, CssToken::Value(buffer.trim().to_owned())));
                    buffer.clear();
                }
                tokens.push(make_token(css_iterator, CssToken::BlockEnd));
            },
            ':' => {
                tokens.push(make_token(css_iterator, CssToken::Property(buffer.trim().to_owned())));
                buffer.clear();
            },
            ';' => {
                if buffer.trim().len() > 0 {
                    tokens.push(make_token(css_iterator, CssToken::Value(buffer.trim().to_owned())));
                    buffer.clear();
                }
            }
            char @ _ => {
                buffer.push(char);
            }
        }
    }
}

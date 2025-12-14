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
    Identifier(String),
    #[allow(unused)] StringLiteral(String), //TODO: implement
    #[allow(unused)] AtKeyword(String), //TODO: implement

    OpenBrace,
    CloseBrace,
    OpenParenthesis,
    CloseParenthesis,

    Dot,
    Plus,
    Greater,
    Tilde,
    Semicolon,
    Comma,
    Colon,
    Hash,

    Whitespace,
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
    let mut last_token_was_whitespace = false;

    'main_loop: while css_iterator.has_next() {

        if in_comment {
            if css_iterator.peek().is_some() {
                match css_iterator.peek().unwrap() {
                    '*' => {
                        css_iterator.next(); //eat the *
                        if css_iterator.peek() == Some(&'/') {
                            css_iterator.next(); //eat the /
                            in_comment = false;
                        }
                    },
                    _ => {
                        //Skip the content in the comment
                        css_iterator.next();
                    }
                }
            }
        } else {
            if css_iterator.peek().is_some() {
                match css_iterator.peek().unwrap() {
                    '/' => {
                        css_iterator.next(); //eat the /
                        if css_iterator.peek() == Some(&'*') {
                            css_iterator.next(); //eat the *
                            in_comment = true;
                        } else {
                            buffer.push('/');
                        }
                    }
                    _ => {}
                }
            }
        }
        if in_comment {
            continue 'main_loop
        }


        match css_iterator.next() {

            char @ ('{' | '}' | '(' | ')' | '.' | '+' | '>' | ',' | ':' | ';' | '~'| '#') => {
                last_token_was_whitespace = false;

                if !buffer.is_empty() {
                    tokens.push(make_token(css_iterator, CssToken::Identifier(buffer.clone())));
                    buffer.clear();
                }

                match char {
                    '{' => tokens.push(make_token(css_iterator, CssToken::OpenBrace)),
                    '}' => tokens.push(make_token(css_iterator, CssToken::CloseBrace)),
                    '(' => tokens.push(make_token(css_iterator, CssToken::OpenParenthesis)),
                    ')' => tokens.push(make_token(css_iterator, CssToken::CloseParenthesis)),
                    '.' => tokens.push(make_token(css_iterator, CssToken::Dot)),
                    '+' => tokens.push(make_token(css_iterator, CssToken::Plus)),
                    '>' => tokens.push(make_token(css_iterator, CssToken::Greater)),
                    ',' => tokens.push(make_token(css_iterator, CssToken::Comma)),
                    ':' => tokens.push(make_token(css_iterator, CssToken::Colon)),
                    ';' => tokens.push(make_token(css_iterator, CssToken::Semicolon)),
                    '~' => tokens.push(make_token(css_iterator, CssToken::Tilde)),
                    '#' => tokens.push(make_token(css_iterator, CssToken::Hash)),

                    _ => {
                        panic!("invalid state");
                    }
                }
            }
            ' ' | '\t' | '\n' => {
                if !buffer.is_empty() {
                    tokens.push(make_token(css_iterator, CssToken::Identifier(buffer.clone())));
                    buffer.clear();
                }

                if !last_token_was_whitespace {
                    tokens.push(make_token(css_iterator, CssToken::Whitespace));
                    last_token_was_whitespace = true;
                }
            }
            '@' => {
                //last_token_was_whitespace = false;  //TODO: enable when this case is implemented
                todo!(); //TODO: read everything after until a non-ident char, and build token
            },
            char @ _ => {
                last_token_was_whitespace = false;
                buffer.push(char);
            }
        }
    }

}

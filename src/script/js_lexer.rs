use crate::html_lexer::TrackingIterator;


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct JsTokenWithLocation {
    pub token: JsToken,
    pub line: u32,
    pub character: u32
}
impl JsTokenWithLocation {
    fn make(js_iterator: &TrackingIterator, token: JsToken) -> JsTokenWithLocation {
        return JsTokenWithLocation { token: token, line: js_iterator.current_line, character: js_iterator.current_char };
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum JsToken {
    Newline,
    Number(String),
    LiteralString(String),
    Identifier(String),
    Dot,
    Equals,
    Semicolon,
    Whitespace,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Plus,
    Minus,
    Star,
    ForwardSlash,
    Comma,
}


pub fn lex_js(document: &str, starting_line: u32, starting_char_idx: u32) -> Vec<JsTokenWithLocation> {
    let mut tokens = Vec::new();

    let mut js_iterator = TrackingIterator {
        iter: document.chars().peekable(),
        current_line: starting_line,
        current_char: starting_char_idx,
    };

    while js_iterator.has_next() {

        if js_iterator.has_next() && js_iterator.peek().unwrap().is_numeric() {
            let mut number_text = String::new();

            while js_iterator.has_next() && js_iterator.peek().unwrap().is_numeric() {
                number_text.push(js_iterator.next());
            }

            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Number(number_text)));
        }
        else if js_iterator.peek() == Some(&' ') || js_iterator.peek() == Some(&'\t') || js_iterator.peek() == Some(&'\r') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Whitespace));
            eat_whitespace(&mut js_iterator);
        }
        else if js_iterator.peek() == Some(&'"') {
            //TODO: this does not account for escaped quotes yet...
            js_iterator.next();

            let mut literal = String::new();
            while js_iterator.has_next() && js_iterator.peek() != Some(&'"') {
                literal.push(js_iterator.next());
            }

            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::LiteralString(literal)));
            js_iterator.next(); //eat the closing "
        }
        else if js_iterator.peek() == Some(&';') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Semicolon));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'=') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Equals));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'+') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Plus));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'-') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Minus));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'*') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Star));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'/') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::ForwardSlash));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&',') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Comma));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'.') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Dot));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'\n') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Newline));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'(') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::OpenParenthesis));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&')') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::CloseParenthesis));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'[') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::OpenBracket));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&']') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::CloseBracket));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'{') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::OpenBrace));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'}') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::CloseBrace));
            js_iterator.next();
        }
        else if js_iterator.peek().is_some() && is_valid_first_char_of_identifier(*js_iterator.peek().unwrap()) {
            let mut identifier = String::new();

            while js_iterator.has_next() && is_valid_identifier_char(*js_iterator.peek().unwrap()) {
                identifier.push(js_iterator.next());
            }

            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Identifier(identifier)));
        }
        else {
            println!("{}", js_iterator.peek().unwrap());
            todo!("unrecognized character in the js tokenizer");
        }
    }

    return tokens;
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


fn is_valid_identifier_char(c: char) -> bool {
    return c.is_alphanumeric() || c == '_' || c == '$';
}


fn is_valid_first_char_of_identifier(c: char) -> bool {
    //the first char of an identifier cannot be a number
    return (c.is_alphanumeric() && !c.is_numeric()) || c == '_' || c == '$';
}


fn is_whitespace(c: char) -> bool {
    //Note that for js, newline is not whitespace (since it has semantics with semicolon insertion)
    return c == ' ' || c == '\t' || c == '\r';
}

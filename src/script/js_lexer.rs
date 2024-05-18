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
    Number(String),
    LiteralString(String),
    Identifier(String),
    RegexLiteral(String),
    Dot,
    Equals,
    Semicolon,
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
    Bigger,
    Smaller,
    And,
    Pipe,
    ExclamationMark,

    //whitespace:
    Newline,
    Whitespace,

    //all keywords:
    KeyWordVar,

    //not an actual token of the language, but used as a way to block out:
    None,
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

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Number(number_text)));
        }
        else if js_iterator.peek() == Some(&' ') || js_iterator.peek() == Some(&'\t') || js_iterator.peek() == Some(&'\r') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Whitespace));
            eat_whitespace(&mut js_iterator);
        }
        else if js_iterator.peek() == Some(&'"') || js_iterator.peek() == Some(&'\'') {
            //TODO: this does not account for escaped quotes yet...
            //TODO: we would need to check which type of quote started this, and only stop on that one, including the other in the string
            js_iterator.next();

            let mut literal = String::new();
            while js_iterator.has_next() && (js_iterator.peek() != Some(&'"') || js_iterator.peek() == Some(&'\'')) {
                literal.push(js_iterator.next());
            }

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
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
            //This is either a token on its own (for division), or it is the start of a literal regex. Figuring this out actually requires
            //  parsing rather then lexing. For now we rely on heuristics as described in
            //  https://stackoverflow.com/questions/5519596/when-parsing-javascript-what-determines-the-meaning-of-a-slash

            //TODO: this can also be a comment, but we should strip those in an earlier pass


            //TODO: put this in a better place where we don't need to instatiate it so often
            const TOKENS_PROBABLY_PRECEDING_REGEX_LITERAL: [JsToken; 13] = [
                JsToken::OpenParenthesis,
                JsToken::Dot,
                JsToken::OpenBracket,
                JsToken::Equals,
                JsToken::Star,
                JsToken::Plus,
                JsToken::Minus,
                JsToken::Semicolon,
                JsToken::Bigger,
                JsToken::Smaller,
                JsToken::And,
                JsToken::Pipe,
                JsToken::ExclamationMark,
                //TODO: when we properly parse multi char operators (like "&&" and "=="), we need to add them to this list
            ];

            let mut last_token = None;
            for token in tokens.iter().rev() {
                if token.token != JsToken::Whitespace && token.token != JsToken::Newline {
                    last_token = Some(token.token.clone());
                    break;
                }
            };

            if last_token.is_none() || (last_token.is_some() && TOKENS_PROBABLY_PRECEDING_REGEX_LITERAL.contains(&last_token.unwrap())) {
                //we are parsing a regex literal

                let mut buffer = String::new();
                buffer.push(js_iterator.next());  // read the opening slash

                'literal_regex_parse: while js_iterator.has_next() {
                    if js_iterator.peek() == Some(&'/') {
                        buffer.push(js_iterator.next());  // read the closing slash

                        //TODO: put this in a better place where we don't need to instatiate it so often
                        const REGEX_ALLOWED_FLAGS: [char; 8] = ['d', 'g', 'i', 'm', 's', 'u', 'v', 'y'];

                        //read possible flags:
                        while js_iterator.has_next() {
                            if js_iterator.peek().is_some() && REGEX_ALLOWED_FLAGS.contains(&js_iterator.peek().unwrap()) {
                                buffer.push(js_iterator.next())
                            } else {
                                break 'literal_regex_parse;
                            }
                        }
                    } else {
                        buffer.push(js_iterator.next());
                    }
                }

                //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::RegexLiteral(buffer)))

            } else {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::ForwardSlash));
                js_iterator.next();
            }

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
        else if js_iterator.peek() == Some(&'>') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Bigger));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'<') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Smaller));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'&') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::And));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'|') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Pipe));
            js_iterator.next();
        }
        else if js_iterator.peek() == Some(&'!') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::ExclamationMark));
            js_iterator.next();
        }
        else if js_iterator.peek().is_some() && is_valid_first_char_of_identifier(*js_iterator.peek().unwrap()) {
            let mut identifier = String::new();

            while js_iterator.has_next() && is_valid_identifier_char(*js_iterator.peek().unwrap()) {
                identifier.push(js_iterator.next());
            }

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
            if identifier == "var" {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::KeyWordVar));
            } else {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Identifier(identifier)));
            }
        }
        else {
            //TODO: when we are confident we have all relevant characters, we should just ignore here (don't give an error, maybe a warning in devconsole)
            todo!("unrecognized character in the js tokenizer: {:?}", js_iterator.peek());
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

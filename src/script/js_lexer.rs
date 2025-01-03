use std::{
    iter::Peekable,
    str::Chars
};

use crate::html_lexer::TrackingIterator;


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct JsTokenWithLocation {
    pub token: JsToken,
    pub line: u32,
    pub character: u32
}
impl JsTokenWithLocation {
    fn make(js_iterator: &JsSourceIterator, token: JsToken) -> JsTokenWithLocation {
        return JsTokenWithLocation { token: token, line: js_iterator.iter.current_line, character: js_iterator.iter.current_char };
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
    Colon,
    QuestionMark,
    BitWiseOr,
    Hash,

    //whitespace:
    Newline,
    Whitespace,

    //all keywords:
    KeyWordVar,
    KeyWordFunction,
    KeyWordReturn,

    //not an actual token of the language, but used as a way to block out:
    None,
}


pub struct JsSourceIterator<'document> {
    //This is a trackingIterator wrapper that treats commented code as whitespace
    //    we do this so we can implement the comment logic in one place, without
    //    allocating new strings as we would do with a pre-process pass.

    pub iter: TrackingIterator<'document>,
    next: Option<char>,
    prev: Option<char>,
    current_string_starter: Option<char>,
}
impl <'document> JsSourceIterator<'document> {
    pub fn new(inner_iter: Peekable<Chars<'document>>, current_line: u32, current_char: u32) -> JsSourceIterator<'document> {
        let iter = TrackingIterator {
            iter: inner_iter,
            current_line,
            current_char,
        };
        return JsSourceIterator {
            iter,
            next: None,
            prev: None,
            current_string_starter: None,
        }
    }
    pub fn has_next(&mut self) -> bool {
        return self.next.is_some() || self.iter.has_next();
    }
    pub fn peek(&mut self) -> Option<char> {
        self.skip_possible_comment();
        if self.next.is_some() {
            return self.next;
        }
        return self.iter.peek().copied();
    }
    pub fn next(&mut self) -> char {
        self.skip_possible_comment();
        let next_char = if self.next.is_some() {
            let c = self.next;
            self.next = None;
            c.unwrap()
        } else {
            self.iter.next()
        };
        if (next_char == '"' || next_char == '\'') && self.prev != Some('\\') {
            if self.current_string_starter.is_none() {
                self.current_string_starter = Some(next_char);
            } else {
                if self.current_string_starter == Some(next_char) { // we check if the string was started with the same kind of quote
                    self.current_string_starter = None;
                }
            }
        }
        self.prev = Some(next_char);
        return next_char;
    }
    fn skip_possible_comment(&mut self) {
        if !self.iter.has_next() {
            return;
        }
        if self.next.is_none() {
            self.next = Some(self.iter.next());
        }
        if self.current_string_starter.is_some() {
            return;  //we are currently reading inside a string, so a comment can't be started
        }

        if self.next == Some('/') && self.iter.peek() == Some(&'/') {
            while self.iter.peek() != Some(&'\n') {
                self.iter.next();
            }
            self.next = None;
        }

        if self.next == Some('/') && self.iter.peek() == Some(&'*') {
            loop {
                self.next = Some(self.iter.next());
                if self.next == Some('*') && self.iter.peek() == Some(&'/') {
                    self.iter.next();
                    self.next = None;
                    break;
                }
            }
        }
    }
}


pub fn lex_js(document: &str, starting_line: u32, starting_char_idx: u32) -> Vec<JsTokenWithLocation> {
    let mut tokens = Vec::new();

    let mut js_iterator = JsSourceIterator::new(document.chars().peekable(), starting_line, starting_char_idx);

    while js_iterator.has_next() {

        if js_iterator.has_next() && js_iterator.peek().unwrap().is_numeric() {
            let mut number_text = String::new();

            while js_iterator.has_next() && js_iterator.peek().unwrap().is_numeric() {
                number_text.push(js_iterator.next());
            }

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Number(number_text)));
        }
        else if js_iterator.peek() == Some(' ') || js_iterator.peek() == Some('\t') || js_iterator.peek() == Some('\r') {
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Whitespace));
            eat_whitespace(&mut js_iterator);
        }
        else if js_iterator.peek() == Some('"') || js_iterator.peek() == Some('\'') || js_iterator.peek() == Some('`') {
            //TODO: this would also match "bla ' " , but by matching the ', not the corresponding "
            //TODO: the backtick is for string tempates and is actually more complicated
            //      see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#tagged_templates

            let quote_type_used = js_iterator.next();
            let mut literal = String::new();
            let mut next_char_is_escaped = false;
            while js_iterator.has_next() && (js_iterator.peek().unwrap() != quote_type_used || next_char_is_escaped) {

                if js_iterator.peek() == Some('\\') {
                    next_char_is_escaped = true;
                    js_iterator.next();
                    continue;
                } else {
                    next_char_is_escaped = false;
                }

                literal.push(js_iterator.next());
            }

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
            tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::LiteralString(literal)));
            js_iterator.next(); //eat the closing "
        }
        else if js_iterator.peek() == Some('/') {
            //This is either a token on its own (for division), or it is the start of a literal regex. Figuring this out actually requires
            //  parsing rather then lexing. For now we rely on heuristics as described in
            //  https://stackoverflow.com/questions/5519596/when-parsing-javascript-what-determines-the-meaning-of-a-slash

            //TODO: put this in a better place where we don't need to instatiate it so often
            const TOKENS_PROBABLY_PRECEDING_REGEX_LITERAL: [JsToken; 14] = [
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
                JsToken::BitWiseOr,
                //TODO: when we properly parse multi char operator tokens (like "&&" and "=="), we need to add them to this list
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

                let mut prev_was_escape_char = false;
                'literal_regex_parse: while js_iterator.has_next() {
                    if js_iterator.peek() == Some('\\') {
                        prev_was_escape_char = true;
                        js_iterator.next();
                        continue;
                    }

                    if !prev_was_escape_char && js_iterator.peek() == Some('/') {
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
                    prev_was_escape_char = false;
                }

                //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::RegexLiteral(buffer)))

            } else {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::ForwardSlash));
                js_iterator.next();
            }

        }
        else if js_iterator.peek().is_some() && is_valid_first_char_of_identifier(js_iterator.peek().unwrap()) {
            let mut identifier = String::new();

            while js_iterator.has_next() && is_valid_identifier_char(js_iterator.peek().unwrap()) {
                identifier.push(js_iterator.next());
            }

            //TODO: using "make" below is not correct, because it will give the end position of the literal, instead of the start
            if identifier == "var" {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::KeyWordVar));
            } else if identifier == "function" {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::KeyWordFunction));
            } else if identifier == "return" {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::KeyWordReturn));
            } else {
                tokens.push(JsTokenWithLocation::make(&js_iterator, JsToken::Identifier(identifier)));
            }
        }
        else {
            //from here we parse single chars as tokens, so any more complex tokens should have been handled before this point

            if js_iterator.peek().is_some() {
                    let next_char = js_iterator.next();

                    let token = match next_char {
                        '(' => { JsToken::OpenParenthesis }
                        ')' => { JsToken::CloseParenthesis }
                        '[' => { JsToken::OpenBracket }
                        ']' => { JsToken::CloseBracket }
                        '{' => { JsToken::OpenBrace }
                        '}' => { JsToken::CloseBrace }
                        ',' => { JsToken::Comma }
                        '.' => { JsToken::Dot }
                        ':' => { JsToken::Colon }
                        ';' => { JsToken::Semicolon }
                        '>' => { JsToken::Bigger }
                        '<' => { JsToken::Smaller }
                        '!' => { JsToken::ExclamationMark }
                        '?' => { JsToken::QuestionMark }
                        '|' => { JsToken::Pipe }
                        '&' => { JsToken::And }
                        '^' => { JsToken::BitWiseOr }
                        '#' => { JsToken::Hash }
                        '=' => { JsToken::Equals }
                        '+' => { JsToken::Plus }
                        '-' => { JsToken::Minus }
                        '*' => { JsToken::Star }

                        '\n' => { JsToken::Newline }

                        _ => {
                            //TODO: when we are confident we have all relevant characters, we should just ignore here (don't give an error, maybe a warning in devconsole)
                            todo!("unrecognized character in the js tokenizer: {:?}", js_iterator.peek());
                        }
                    };

                    tokens.push(JsTokenWithLocation::make(&js_iterator, token));
            }

        }
    }

    return tokens;
}


fn eat_whitespace(iterator: &mut JsSourceIterator) {
    loop {
        let opt_peek = iterator.peek();
        if opt_peek.is_none() {
            return
        }
        if is_whitespace(opt_peek.unwrap()) {
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

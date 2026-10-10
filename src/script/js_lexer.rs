use std::iter::Peekable;
use std::str::Chars;

use crate::tracking_iterator::TrackingIterator;


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub struct JsTokenWithLocation {
    pub token: JsToken,
    pub line: u32,
    pub character: u32,
    pub had_newline_before: bool,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum JsToken {
    Number(f64),
    LiteralString(String),
    LiteralBoolean(bool),
    LiteralUndefined,
    Identifier(String),
    RegexLiteral(String),
    Dot,
    Assign,
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
    ExclamationMark,
    Colon,
    QuestionMark,
    BitWiseOr,
    BitWiseXor,
    BitWiseAnd,
    Hash,
    LogicalAnd,
    LogicalOr,
    RightShift,
    LeftShift,
    UnsignedRightShift,
    Increment,
    Decrement,
    Remainder,

    Bigger,
    Smaller,
    Equals,
    EqualsStrict,
    NotEquals,
    NotEqualsStrict,
    BiggerOrEqual,
    SmallerOrEqual,

    //compound assignment operators:
    CompoundAssignAdd,
    CompoundAssignMinus,
    CompoundAssignTimes,
    CompoundAssignDiv,
    CompoundAssignBitWiseOr,
    CompoundAssignBitWiseXor,
    CompoundAssignBitWiseAnd,

    //all keywords:
    KeyWordVar,
    KeyWordLet,
    KeyWordConst,
    KeyWordFunction,
    KeyWordReturn,
    KeyWordIf,
    KeyWordElse,
    KeyWordNew,
    KeyWordWhile,
    KeyWordFor,
    KeyWordTypeOf,
    KeyWordIn,
    KeyWordTry,
    KeyWordCatch,
    KeyWordFinally,
    KeyWordThrow,
    KeyWordDelete,
}


pub struct JsSourceIterator<'document> {
    //This is a trackingIterator wrapper that treats commented code as whitespace
    //    we do this so we can implement the comment logic in one place, without
    //    allocating new strings as we would do with a pre-process pass.

    pub iter: TrackingIterator<'document>,
    next: Option<char>,
    prev: Option<char>,
    current_string_starter: Option<char>,
    in_regex_literal: bool,
    new_line_pending: bool,
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
            in_regex_literal: false,
            new_line_pending: false,
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
            return; //we are currently reading inside a string, so a comment can't be started
        }
        if self.in_regex_literal {
            return; //we are reading inside a literal regex, and // is just text to match there, not a comment
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


const TOKENS_PROBABLY_PRECEDING_REGEX_LITERAL: &[JsToken] = &[
    JsToken::OpenParenthesis,
    JsToken::OpenBracket,
    JsToken::Assign,
    JsToken::CompoundAssignAdd,
    JsToken::CompoundAssignMinus,
    JsToken::CompoundAssignTimes,
    JsToken::CompoundAssignDiv,
    JsToken::CompoundAssignBitWiseOr,
    JsToken::CompoundAssignBitWiseXor,
    JsToken::CompoundAssignBitWiseAnd,
    JsToken::Star,
    JsToken::Plus,
    JsToken::Minus,
    JsToken::Semicolon,
    JsToken::Bigger,
    JsToken::Smaller,
    JsToken::BitWiseOr,
    JsToken::BitWiseXor,
    JsToken::BitWiseAnd,
    JsToken::ExclamationMark,
    JsToken::BitWiseOr,
    JsToken::Equals,
    JsToken::EqualsStrict,
    JsToken::NotEquals,
    JsToken::NotEqualsStrict,
    JsToken::BiggerOrEqual,
    JsToken::SmallerOrEqual,
    JsToken::LogicalAnd,
    JsToken::LogicalOr,
    JsToken::KeyWordReturn,
    JsToken::Comma,
    JsToken::Colon,
    JsToken::QuestionMark,
    JsToken::RightShift,
    JsToken::LeftShift,
    JsToken::UnsignedRightShift,
    JsToken::KeyWordTypeOf,
    JsToken::KeyWordDelete,
];


const REGEX_ALLOWED_FLAGS: &[char] = &['d', 'g', 'i', 'm', 's', 'u', 'v', 'y'];


pub fn lex_js(document: &str, starting_line: u32, starting_char_idx: u32) -> Vec<JsTokenWithLocation> {
    let mut tokens = Vec::new();

    let mut js_iterator = JsSourceIterator::new(document.chars().peekable(), starting_line, starting_char_idx);

    while js_iterator.has_next() {

        if js_iterator.peek().unwrap() == '\n' {
            js_iterator.next();
            js_iterator.new_line_pending = true;
            continue;
        }

        if js_iterator.has_next() && js_iterator.peek().unwrap().is_numeric() {
            tokens.push(lex_number(&mut js_iterator));
        }
        else if js_iterator.peek() == Some(' ') || js_iterator.peek() == Some('\t') || js_iterator.peek() == Some('\r') {
            eat_whitespace(&mut js_iterator);
        }
        else if js_iterator.peek() == Some('"') || js_iterator.peek() == Some('\'') || js_iterator.peek() == Some('`') {
            //TODO: this would also match "bla ' " , but by matching the ', not the corresponding "
            //TODO: the backtick is for string tempates and is actually more complicated
            //      see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#tagged_templates

            let token_line_pos = js_iterator.iter.current_line;
            let token_char_pos = js_iterator.iter.current_char;

            let quote_type_used = js_iterator.next();
            let mut literal = String::new();
            let mut next_char_is_escaped = false;
            while js_iterator.has_next() && (js_iterator.peek().unwrap() != quote_type_used || next_char_is_escaped) {

                if js_iterator.peek() == Some('\\') && !next_char_is_escaped {
                    next_char_is_escaped = true;
                    js_iterator.next();
                    continue;
                } else {
                    next_char_is_escaped = false;
                }

                literal.push(js_iterator.next());
            }

            let token = JsTokenWithLocation { token: JsToken::LiteralString(literal), line: token_line_pos,
                                              character: token_char_pos, had_newline_before: js_iterator.new_line_pending };
            js_iterator.new_line_pending = false;
            tokens.push(token);
            js_iterator.next(); //eat the closing "
        }
        else if js_iterator.peek() == Some('/') {
            //This is either a token on its own (for division), or it is the start of a literal regex. Figuring this out actually requires
            //  parsing rather then lexing. For now we rely on heuristics as described in
            //  https://stackoverflow.com/questions/5519596/when-parsing-javascript-what-determines-the-meaning-of-a-slash

            let last_token = Some(tokens.iter().last().unwrap().token.clone());
            let token_line_pos = js_iterator.iter.current_line;
            let token_char_pos = js_iterator.iter.current_char;

            if last_token.is_none() || (last_token.is_some() && TOKENS_PROBABLY_PRECEDING_REGEX_LITERAL.contains(&last_token.unwrap())) {
                js_iterator.in_regex_literal = true;
                let mut buffer = String::new();
                let mut escaped = false;
                let mut in_character_class = false;

                buffer.push(js_iterator.next());  // read the opening slash

                'literal_regex_parse: while js_iterator.has_next() {

                    let ch = js_iterator.next();
                    buffer.push(ch);

                    if escaped {
                        escaped = false;
                        continue;
                    }

                    match ch {
                        '\\' => { escaped = true; }
                        '[' => { in_character_class = true; }
                        ']' => { in_character_class = false; }
                        '/' if !in_character_class => {
                            while js_iterator.has_next() {
                                if let Some(flag) = js_iterator.peek() {
                                    if REGEX_ALLOWED_FLAGS.contains(&flag) {
                                        buffer.push(js_iterator.next());
                                    } else {
                                        break;
                                    }
                                }
                            }
                            break 'literal_regex_parse;
                        },
                        _ => {}
                    }
                }
                js_iterator.in_regex_literal = false;

                let token = JsTokenWithLocation { token: JsToken::RegexLiteral(buffer), line: token_line_pos,
                                                  character: token_char_pos, had_newline_before: js_iterator.new_line_pending };
                js_iterator.new_line_pending = false;
                tokens.push(token);

            } else {
                js_iterator.next();

                if js_iterator.has_next() {
                    match js_iterator.peek().unwrap() {
                        '=' => {
                            js_iterator.next();
                            let token = JsTokenWithLocation { token: JsToken::CompoundAssignDiv, line: token_line_pos,
                                                              character: token_char_pos, had_newline_before: js_iterator.new_line_pending };
                            js_iterator.new_line_pending = false;
                            tokens.push(token);
                            continue;
                        },
                        _ => {}
                    }
                }

                let token = JsTokenWithLocation { token: JsToken::ForwardSlash, line: token_line_pos,
                                                  character: token_char_pos, had_newline_before: js_iterator.new_line_pending };
                js_iterator.new_line_pending = false;
                tokens.push(token);
            }

        }
        else if js_iterator.peek().is_some() && is_valid_first_char_of_identifier(js_iterator.peek().unwrap()) {
            let mut identifier = String::new();

            let line = js_iterator.iter.current_line;
            let character = js_iterator.iter.current_char;

            while js_iterator.has_next() && is_valid_identifier_char(js_iterator.peek().unwrap()) {
                identifier.push(js_iterator.next());
            }

            if identifier == "var" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordVar, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "let" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordLet, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "const" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordConst, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "function" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordFunction, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "return" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordReturn, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "if" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordIf, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "else" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordElse, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "new" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordNew, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "while" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordWhile, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "for" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordFor, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "true" {
                tokens.push(JsTokenWithLocation { token: JsToken::LiteralBoolean(true), line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "false" {
                tokens.push(JsTokenWithLocation { token: JsToken::LiteralBoolean(false), line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "typeof" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordTypeOf, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "in" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordIn, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "undefined" {
                tokens.push(JsTokenWithLocation { token: JsToken::LiteralUndefined, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "try" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordTry, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "catch" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordCatch, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "finally" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordFinally, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "throw" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordThrow, line, character, had_newline_before: js_iterator.new_line_pending });
            } else if identifier == "delete" {
                tokens.push(JsTokenWithLocation { token: JsToken::KeyWordDelete, line, character, had_newline_before: js_iterator.new_line_pending });
            } else {
                tokens.push(JsTokenWithLocation { token: JsToken::Identifier(identifier), line, character, had_newline_before: js_iterator.new_line_pending });
            }
            js_iterator.new_line_pending = false;
        }
        else {
            //from here we parse hardcoded sets of chars as tokens, so any more complex tokens should have been handled before this point

            let line = js_iterator.iter.current_line;
            let character = js_iterator.iter.current_char;

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
                    '%' => { JsToken::Remainder }
                    '>' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '>' => {
                                    js_iterator.next();
                                    if js_iterator.has_next() {
                                        match js_iterator.peek().unwrap() {
                                            '>' => { js_iterator.next(); JsToken::UnsignedRightShift }
                                            _ => { JsToken::RightShift }
                                        }
                                    } else {
                                        JsToken::RightShift
                                    }
                                },
                                '=' => { js_iterator.next(); JsToken::BiggerOrEqual }
                                _ => { JsToken::Bigger }
                            }
                        } else { JsToken::Bigger }
                    },
                    '<' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '<' => { js_iterator.next(); JsToken::LeftShift }
                                '=' => { js_iterator.next(); JsToken::SmallerOrEqual }
                                _ => { JsToken::Smaller }
                            }
                        } else { JsToken::Smaller }
                    },
                    '!' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => {
                                    js_iterator.next();
                                    if js_iterator.has_next() {
                                        match js_iterator.peek().unwrap() {
                                            '=' => { js_iterator.next(); JsToken::NotEqualsStrict },
                                            _ => { JsToken::NotEquals },
                                        }
                                    } else {
                                        JsToken::NotEquals
                                    }
                                }
                                _ => { JsToken::ExclamationMark }
                            }
                        } else { JsToken::ExclamationMark }
                    },
                    '?' => { JsToken::QuestionMark }
                    '^' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => { js_iterator.next(); JsToken::CompoundAssignBitWiseXor }
                                _ => { JsToken::BitWiseXor }
                            }
                        } else { JsToken::BitWiseXor }
                        }
                    '#' => { JsToken::Hash }
                    '+' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => { js_iterator.next(); JsToken::CompoundAssignAdd }
                                '+' => { js_iterator.next(); JsToken::Increment }
                                _ => { JsToken::Plus }
                            }
                        } else { JsToken::Plus }
                    },
                    '-' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => { js_iterator.next(); JsToken::CompoundAssignMinus }
                                '-' => { js_iterator.next(); JsToken::Decrement }
                                _ => { JsToken::Minus }
                            }
                        } else { JsToken::Minus }
                    },
                    '*' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => { js_iterator.next(); JsToken::CompoundAssignTimes }
                                _ => { JsToken::Star }
                            }
                        } else { JsToken::Star }
                    },
                    '|' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '|' => { js_iterator.next(); JsToken::LogicalOr }
                                '=' => { js_iterator.next(); JsToken::CompoundAssignBitWiseOr }
                                _ => { JsToken::BitWiseOr }
                            }
                        } else { JsToken::BitWiseOr }
                    },
                    '&' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '&' => { js_iterator.next(); JsToken::LogicalAnd }
                                '=' => { js_iterator.next(); JsToken::CompoundAssignBitWiseAnd }
                                _ => { JsToken::BitWiseAnd }
                            }
                        } else { JsToken::BitWiseAnd }
                    },
                    '=' => {
                        if js_iterator.has_next() {
                            match js_iterator.peek().unwrap() {
                                '=' => {
                                    js_iterator.next();
                                    if js_iterator.has_next() {
                                        match js_iterator.peek().unwrap() {
                                            '=' => { js_iterator.next(); JsToken::EqualsStrict },
                                            _ => { JsToken::Equals },
                                        }
                                    } else {
                                        JsToken::Equals
                                    }
                                }
                                _ => { JsToken::Assign }
                            }
                        } else { JsToken::Assign }
                    },

                    _ => {
                        //TODO: when we are confident we have all relevant characters, we should just ignore here (don't give an error, maybe a warning in devconsole)
                        todo!("unrecognized character in the js tokenizer: {:?}", next_char);
                    }
                };

                let token = JsTokenWithLocation { token, line, character, had_newline_before: js_iterator.new_line_pending };
                js_iterator.new_line_pending = false;
                tokens.push(token);
            }
        }
    }

    return tokens;
}


fn lex_number(js_iterator: &mut JsSourceIterator) -> JsTokenWithLocation {
    let token_line_pos = js_iterator.iter.current_line;
    let token_char_pos = js_iterator.iter.current_char;

    let mut number_text = String::new();

    number_text.push(js_iterator.next());

    let number_value = if number_text == "0" && js_iterator.has_next() && js_iterator.peek().unwrap() == 'x' {
        //Hexadecimal number
        number_text.clear();
        js_iterator.next();

        while js_iterator.has_next() && js_iterator.peek().unwrap().is_ascii_hexdigit() {
            number_text.push(js_iterator.next());
        }

        i64::from_str_radix(number_text.as_str(), 16).unwrap() as f64

    } else {
        while js_iterator.has_next() {
            if js_iterator.peek().unwrap().is_numeric() ||
                    js_iterator.peek().unwrap() == '.' ||
                    js_iterator.peek().unwrap() == 'e' { //the exponential notation will just be converted when converting the string to an int
                number_text.push(js_iterator.next());
            } else {
                break;
            }
        }

        number_text.parse().unwrap()
    };

    let token = JsTokenWithLocation { token: JsToken::Number(number_value), line: token_line_pos,
                                      character: token_char_pos, had_newline_before: js_iterator.new_line_pending };
    js_iterator.new_line_pending = false;
    return token;
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

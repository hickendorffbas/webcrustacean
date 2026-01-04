#[derive(Debug, PartialEq)]
pub enum Token {
    //TODO: there are probably more places I can use anonymous enum structs
    StartTag {
        name: String,
        self_closing: bool, //TODO: is this ever really used?
    },
    EndTag {
        name: String,
    },
    Text(String),
    EOF,
}

#[derive(Debug)]
enum HtmlLexerState {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    EndTagName,
}

pub struct Lexer {
    input: String,
    position: usize,
    state: HtmlLexerState,
    buffer: String,
}
impl Lexer {
    pub fn new(input: String) -> Self {
        return Lexer {
            input,
            position: 0,
            state: HtmlLexerState::Data,
            buffer: String::new(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?; //TODO: is it not way more efficient to store the chars iterator on the state?
        self.position += ch.len_utf8();
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let ch = match self.next_char() {
                Some(c) => c,
                None => {
                    if !self.buffer.is_empty() {
                        match self.state {
                            HtmlLexerState::Data => {
                                let text = std::mem::take(&mut self.buffer);
                                return Token::Text(text);
                            },
                            HtmlLexerState::TagOpen => todo!(), //TODO: we probably just ignore this and return EOF?
                            HtmlLexerState::EndTagOpen => todo!(), //TODO: we probably just ignore this and return EOF?
                            HtmlLexerState::TagName => todo!(), //TODO: we probably just ignore this and return EOF?
                            HtmlLexerState::EndTagName => todo!(), //TODO: we probably just ignore this and return EOF?
                        }
                    }
                    return Token::EOF;
                }
            };

            match self.state {
                HtmlLexerState::Data => {
                    if ch == '<' {
                        self.state = HtmlLexerState::TagOpen;
                        if !self.buffer.is_empty() {
                            let text = std::mem::take(&mut self.buffer);
                            return Token::Text(text);
                        }
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::TagOpen => {
                    if ch == '/' {
                        self.state = HtmlLexerState::EndTagOpen;
                    } else {
                        self.buffer.push(ch);
                        self.state = HtmlLexerState::TagName;
                    }
                },
                HtmlLexerState::EndTagOpen => {
                    self.buffer.push(ch);
                    self.state = HtmlLexerState::EndTagName;
                },
                HtmlLexerState::TagName => {
                    if ch.is_whitespace() || ch == '>' {  //TODO: the whitespace here means we need to go to parse attributes
                        let name = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::Data;

                        if self.peek_char() == Some('/') {
                            self.next_char();
                            self.next_char(); // eat the ">" //TODO: does this work?
                            return Token::StartTag { name, self_closing: true, };
                        }

                        return Token::StartTag { name, self_closing: false };
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::EndTagName => {
                    if ch == '>' {
                        let name = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::Data;
                        return Token::EndTag { name };
                    } else {
                        self.buffer.push(ch);
                    }

                }
            }
        }
    }
}

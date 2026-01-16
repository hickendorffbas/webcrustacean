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
    Attribute {
        name: String,
        value: String,
    },
    Doctype {
        content: String,
    },
    EOF,
}


#[derive(Debug)]
enum HtmlLexerState {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    InTag,
    EndTagName,
    AttributeName,
    AttributeValueStart,
    AttributeValue,
    EntityInData,
    EntityInAttributeValue,
    InComment,
    InDocType,
}


#[derive(Debug, PartialEq)]
enum InQuotes {
    None,
    Single,
    Double,
}


pub struct Lexer {
    input: String,
    position: usize,
    state: HtmlLexerState,
    buffer: String,
    entity_buffer: String,
    open_attribute_name: Option<String>,
    in_quotes: InQuotes,
}
impl Lexer {
    pub fn new(input: String) -> Self {
        return Lexer {
            input,
            position: 0,
            state: HtmlLexerState::Data,
            buffer: String::new(),
            entity_buffer: String::new(),
            open_attribute_name: None,
            in_quotes: InQuotes::None,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?; //TODO: is it not way more efficient to store the chars (peekable) iterator on the state?
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
                            _ => todo!(), //TODO: we probably just ignore this and return EOF?
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
                    } else if ch == '&' {
                        self.state = HtmlLexerState::EntityInData;
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
                    if ch == 'E' && self.buffer == "!DOCTYP" {
                        self.state = HtmlLexerState::InDocType;
                    } else if ch == '-' && self.buffer == "!-" {
                        self.state = HtmlLexerState::InComment;
                        self.buffer = String::new();
                    } else if ch.is_whitespace() {
                        let name = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::InTag;

                        return Token::StartTag { name, self_closing: false };
                    } else if ch == '>' {
                        let name = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::Data;
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
                },
                HtmlLexerState::InTag => {
                    if ch.is_whitespace() {
                        //we do nothing here, we wait for more content
                    } else if ch == '>' {
                        self.state = HtmlLexerState::Data;
                    } else {
                        self.state = HtmlLexerState::AttributeName;
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::AttributeName => {
                    if ch.is_whitespace() {
                        self.state = HtmlLexerState::InTag;

                        let attribute_name = std::mem::take(&mut self.buffer);

                        //for boolean attributes, the value is specced to be equal to the name
                        return Token::Attribute { name: attribute_name.clone(), value: attribute_name };
                    } else if ch == '>' {
                        self.state = HtmlLexerState::Data;

                        let attribute_name = std::mem::take(&mut self.buffer);

                        //for boolean attributes, the value is specced to be equal to the name
                        return Token::Attribute { name: attribute_name.clone(), value: attribute_name };
                    } else if ch == '=' {
                        self.open_attribute_name = Some(std::mem::take(&mut self.buffer));
                        self.state = HtmlLexerState::AttributeValueStart;
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::AttributeValueStart => {
                    if ch.is_whitespace() {
                        //We do nothing, we wait to see an actual value
                    } else if ch == '\'' {
                        self.in_quotes = InQuotes::Single;
                        self.state = HtmlLexerState::AttributeValue;
                    } else if ch == '"' {
                        self.in_quotes = InQuotes::Double;
                        self.state = HtmlLexerState::AttributeValue;
                    } else {
                        self.in_quotes = InQuotes::None;
                        self.buffer.push(ch);
                        self.state = HtmlLexerState::AttributeValue;
                    }
                }
                HtmlLexerState::AttributeValue => {
                    if (ch == '\'' && self.in_quotes == InQuotes::Single) ||
                       (ch == '"' && self.in_quotes == InQuotes::Double) {

                        self.state = HtmlLexerState::InTag;

                        let token = Token::Attribute {
                            name: std::mem::take(&mut self.open_attribute_name).unwrap(),
                            value: std::mem::take(&mut self.buffer),
                        };
                        self.open_attribute_name = None;
                        return token;
                    } else if ch == '&' {
                        self.state = HtmlLexerState::EntityInAttributeValue;
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::EntityInData => {
                    if ch == ';' {
                        self.push_entity();
                        self.state = HtmlLexerState::Data;
                    } else {
                        self.entity_buffer.push(ch);
                    }
                },
                HtmlLexerState::EntityInAttributeValue => {
                    if ch == ';' {
                        self.push_entity();
                        self.state = HtmlLexerState::AttributeValue;
                    } else {
                        self.entity_buffer.push(ch);
                    }
                },
                HtmlLexerState::InComment => {
                    if ch == '>' && self.buffer.ends_with("--") {
                        self.buffer = String::new();
                        self.state = HtmlLexerState::Data;
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::InDocType => {
                    if ch == '>' {
                        let doctype_content = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::Data;
                        return Token::Doctype { content: doctype_content };
                    } else {
                        self.buffer.push(ch);
                    }
                }
            }
        }
    }

    fn push_entity(&mut self) {
        let entity_text = std::mem::take(&mut self.entity_buffer);

        match entity_text.as_str() {
            "amp" => self.buffer.push('&'),
            "gt" =>  self.buffer.push('>'),
            "lt" =>  self.buffer.push('<'),
            "quot" =>  self.buffer.push('"'),
            //TODO: add more entities
            _ => {
                todo!(); //TODO: push the whole entity into the document as text?
            }
        }
    }
}

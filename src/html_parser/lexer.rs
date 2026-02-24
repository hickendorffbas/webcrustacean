#[derive(Debug, PartialEq)]
pub enum Token {
    //TODO: there are probably more places I can use anonymous enum structs
    StartTag {
        name: String,
        self_closing: bool,
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

pub struct TokenWithLocation {
    pub token: Token,
    pub line: usize,
    pub char: usize,
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
    InScript,
    InStyle,
}


#[derive(Debug, PartialEq)]
enum InQuotes {
    None,
    Single,
    Double,
}


static SELF_CLOSING_TAGS: &[&str] = &[
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "keygen",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr",
];


pub struct Lexer {
    input: String,
    position: usize,
    state: HtmlLexerState,
    buffer: String,
    entity_buffer: String,
    open_attribute_name: Option<String>,
    in_quotes: InQuotes,
    last_tag_name: String,
    line_pos: usize,
    char_pos: usize,
    pending_token: Option<TokenWithLocation>,
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
            last_tag_name: String::new(),
            line_pos: 0,
            char_pos: 0,
            pending_token: None,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?; //TODO: is it not way more efficient to store the chars (peekable) iterator on the state?
                                                              //      seems we don't even need peekable
        self.position += ch.len_utf8();

        if ch == '\n' {
            self.line_pos += 1;
            self.char_pos = 0;
        } else {
            self.char_pos += 1;
        }

        Some(ch)
    }

    pub fn make_token(&self, token: Token) -> TokenWithLocation {
        return TokenWithLocation { token, line: self.line_pos, char: self.char_pos };
    }

    pub fn next_token(&mut self) -> TokenWithLocation {
        if self.pending_token.is_some() {
            let pending_token = std::mem::take(&mut self.pending_token);
            return pending_token.unwrap();
        }

        loop {
            let ch = match self.next_char() {
                Some(c) => c,
                None => {
                    if !self.buffer.is_empty() {
                        match self.state {
                            HtmlLexerState::Data => {
                                let text = std::mem::take(&mut self.buffer);
                                return self.make_token(Token::Text(text));
                            },
                            _ => todo!(), //TODO: we probably just ignore this and return EOF?
                        }
                    }
                    return self.make_token(Token::EOF);
                }
            };

            match self.state {
                HtmlLexerState::Data => {
                    if ch == '<' {
                        self.state = HtmlLexerState::TagOpen;
                        if !self.buffer.is_empty() {
                            let text = std::mem::take(&mut self.buffer);
                            return self.make_token(Token::Text(text));
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
                        self.last_tag_name = name.clone();
                        return self.make_token(Token::StartTag { self_closing: is_self_closing(&name), name });
                    } else if ch == '>' {
                        let name = std::mem::take(&mut self.buffer);
                        if name == "script" {
                            self.state = HtmlLexerState::InScript;
                        } else if name == "style" {
                            self.state = HtmlLexerState::InStyle;
                        } else {
                            self.state = HtmlLexerState::Data;
                        }
                        self.last_tag_name = name.clone();
                        return self.make_token(Token::StartTag { self_closing: is_self_closing(&name), name });
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::EndTagName => {
                    if ch == '>' {
                        let name = std::mem::take(&mut self.buffer);
                        self.state = HtmlLexerState::Data;
                        return self.make_token(Token::EndTag { name });
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::InTag => {
                    if ch.is_whitespace() {
                        //we do nothing here, we wait for more content
                    } else if ch == '>' {
                        if self.last_tag_name == "script" {
                            self.state = HtmlLexerState::InScript;
                        } else if self.last_tag_name == "style" {
                            self.state = HtmlLexerState::InStyle;
                        } else {
                            self.state = HtmlLexerState::Data;
                        }
                    } else if ch == '/' {
                        //This does not mean its a closing tag, that case has been handled in TagOpen already
                        //     this is the slash some people put in self-closing tags (even though its not needed), so we ignore it
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
                        return self.make_token(Token::Attribute { name: attribute_name.clone(), value: attribute_name });
                    } else if ch == '>' {
                        self.state = HtmlLexerState::Data;

                        let attribute_name = std::mem::take(&mut self.buffer);

                        //for boolean attributes, the value is specced to be equal to the name
                        return self.make_token(Token::Attribute { name: attribute_name.clone(), value: attribute_name });
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
                       (ch == '"' && self.in_quotes == InQuotes::Double) ||
                       ((ch == ' ' || ch == '>') && self.in_quotes == InQuotes::None) {

                        if ch == '>' {
                            self.state = HtmlLexerState::Data;
                        } else {
                            self.state = HtmlLexerState::InTag;
                        }

                        let token = Token::Attribute {
                            name: std::mem::take(&mut self.open_attribute_name).unwrap(),
                            value: std::mem::take(&mut self.buffer),
                        };
                        self.open_attribute_name = None;
                        return self.make_token(token);
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
                        return self.make_token(Token::Doctype { content: doctype_content });
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::InScript => {
                    //TODO: we need to track quotes, otherwise this will fail: <script>var x = "</script>";</script>
                    if ch == '>' && self.buffer.ends_with("</script") {
                        let mut raw_text_content = std::mem::take(&mut self.buffer);
                        raw_text_content = raw_text_content[..raw_text_content.len() - "</script".len()].to_string();
                        self.state = HtmlLexerState::Data;
                        self.pending_token = Some(self.make_token(Token::EndTag { name: "script".to_owned() }));
                        return self.make_token(Token::Text(raw_text_content));
                    } else {
                        self.buffer.push(ch);
                    }
                },
                HtmlLexerState::InStyle => {
                    if ch == '>' && self.buffer.ends_with("</style") {
                        let mut raw_text_content = std::mem::take(&mut self.buffer);
                        raw_text_content = raw_text_content[..raw_text_content.len() - "</style".len()].to_string();
                        self.state = HtmlLexerState::Data;
                        self.pending_token = Some(self.make_token(Token::EndTag { name: "style".to_owned() }));
                        return self.make_token(Token::Text(raw_text_content));
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
            "amp" =>  self.buffer.push('&'),
            "apos" => self.buffer.push('\''),
            "gt" =>   self.buffer.push('>'),
            "lt" =>   self.buffer.push('<'),
            "nbsp" => self.buffer.push(' '),
            "quot" => self.buffer.push('"'),
            _ => {
                self.buffer.push_str(entity_text.as_str());
            }
        }
    }
}


fn is_self_closing(tagname: &String) -> bool {
    return SELF_CLOSING_TAGS.contains(&tagname.as_str());
}
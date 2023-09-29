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
}


pub fn lex_css(document: &str, starting_line: u32, starting_char_idx: u32) -> Vec<CssTokenWithLocation> {
    let mut tokens = Vec::new();

    //TODO: handle comments

    let mut css_iterator = TrackingIterator {
        iter: document.chars().peekable(),
        current_line: starting_line,
        current_char: starting_char_idx,
    };

    while css_iterator.has_next() {
        lex_css_rule(&mut css_iterator, &mut tokens);
        eat_whitespace(&mut css_iterator);
    }

    return tokens;
}


fn lex_css_rule(css_iterator: &mut TrackingIterator, tokens: &mut Vec<CssTokenWithLocation>) {
    eat_whitespace(css_iterator);

    while css_iterator.has_next() && css_iterator.peek() != Some(&'{') {

        let mut selector_data = String::new();
        while css_iterator.has_next() && css_iterator.peek() != Some(&' ') && css_iterator.peek() != Some(&'{') {
            selector_data.push(css_iterator.next());
        }
        let token = CssToken::Selector(selector_data);
        tokens.push(CssTokenWithLocation { css_token: token, line: css_iterator.current_line, character: css_iterator.current_char });

        eat_whitespace(css_iterator);
    }

    if css_iterator.peek() == Some(&'{') {
        css_iterator.next(); //read the {
        tokens.push(CssTokenWithLocation { css_token: CssToken::BlockStart, line: css_iterator.current_line, character: css_iterator.current_char });

        lex_css_block(css_iterator, tokens);

    }
}


fn lex_css_block(css_iterator: &mut TrackingIterator, tokens: &mut Vec<CssTokenWithLocation>) {
    eat_whitespace(css_iterator);

    //TODO: this function is currently wrong, because new selectors might be nested in the block, and we assume everything is a property

    while css_iterator.has_next() && css_iterator.peek() != Some(&'}') {

        {
            let mut property_data = String::new();
            while css_iterator.has_next() && css_iterator.peek() != Some(&':') {
                property_data.push(css_iterator.next());
            }
            tokens.push(CssTokenWithLocation { css_token: CssToken::Property(property_data),
                                            line: css_iterator.current_line,
                                            character: css_iterator.current_char });
        }

        css_iterator.next(); //read the ":"
        eat_whitespace(css_iterator);

        {
            let mut value_data = String::new();
            while css_iterator.has_next() && css_iterator.peek() != Some(&';') {
                value_data.push(css_iterator.next());
            }
            tokens.push(CssTokenWithLocation { css_token: CssToken::Value(value_data),
                                            line: css_iterator.current_line,
                                            character: css_iterator.current_char });
        }

        css_iterator.next(); //read the ";"
        eat_whitespace(css_iterator);

    }

    if css_iterator.has_next() {
        css_iterator.next(); //read the }
        tokens.push(CssTokenWithLocation { css_token: CssToken::BlockEnd, line: css_iterator.current_line, character: css_iterator.current_char });
    }
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


fn is_whitespace(c: char) -> bool {
    return c == ' ' || c == '\n' || c == '\t' || c == '\r';
}

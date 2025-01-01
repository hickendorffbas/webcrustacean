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

    //TODO: handle comments (maybe reuse the iterator implementation that I have for javascript?)

    let mut css_iterator = TrackingIterator {
        iter: document.chars().peekable(),
        current_line: starting_line,
        current_char: starting_char_idx,
    };

    while css_iterator.has_next() {
        lex_css_block(&mut css_iterator, &mut tokens);
        eat_whitespace(&mut css_iterator);
    }

    return tokens;
}


fn lex_css_block(css_iterator: &mut TrackingIterator, tokens: &mut Vec<CssTokenWithLocation>) {
    'main_loop: while css_iterator.has_next() {
        eat_whitespace(css_iterator);

        if css_iterator.peek() == Some(&'}') {
            //this can happen if we have { } after a selector, or if the last property had a trailing ;
            css_iterator.next(); //eat the }
            tokens.push(CssTokenWithLocation { css_token: CssToken::BlockEnd, line: css_iterator.current_line, character: css_iterator.current_char });
            break 'main_loop;
        }

        let mut selector_or_property_data = String::new();
        while css_iterator.has_next() && css_iterator.peek() != Some(&'{') && css_iterator.peek() != Some(&':') {
            selector_or_property_data.push(css_iterator.next());
        }

        if css_iterator.peek() == Some(&'{') {
            //we have been reading a selector

            css_iterator.next(); //eat the {

            let token = CssToken::Selector(selector_or_property_data.trim().to_owned());
            tokens.push(CssTokenWithLocation { css_token: token, line: css_iterator.current_line, character: css_iterator.current_char });
            tokens.push(CssTokenWithLocation { css_token: CssToken::BlockStart,line: css_iterator.current_line, character: css_iterator.current_char });

            lex_css_block(css_iterator, tokens);
            eat_whitespace(css_iterator);
            if !css_iterator.has_next() {
                break 'main_loop;
            }

        } else if css_iterator.peek() == Some(&':') {
            //we have been reading a property

            css_iterator.next(); //eat the :

            tokens.push(CssTokenWithLocation { css_token: CssToken::Property(selector_or_property_data.trim().to_owned()),
                                               line: css_iterator.current_line,
                                               character: css_iterator.current_char });

            let mut value_data = String::new();
            while css_iterator.has_next() && css_iterator.peek() != Some(&';') && css_iterator.peek() != Some(&'}') {
                value_data.push(css_iterator.next());
            }

            tokens.push(CssTokenWithLocation { css_token: CssToken::Value(value_data.trim().to_owned()), line: css_iterator.current_line, character: css_iterator.current_char });

            if css_iterator.peek() == Some(&';') {
                //this was the rule, we might have another

                css_iterator.next(); //eat the ;

            } else if css_iterator.peek() == Some(&'}') {
                //we are done with this block

                css_iterator.next(); //eat the }

                tokens.push(CssTokenWithLocation { css_token: CssToken::BlockEnd, line: css_iterator.current_line, character: css_iterator.current_char });

                break 'main_loop;
            }
        }
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

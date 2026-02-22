use std::iter::Peekable;
use std::str::Chars;


pub struct TrackingIterator<'document> {
    pub iter: Peekable<Chars<'document>>,
    pub current_line: u32,
    pub current_char: u32,
}
impl TrackingIterator<'_> {
    pub fn next(&mut self) -> char {
        let next_char = self.iter.next().unwrap();
        if next_char == '\n' {
            self.current_line += 1;
            self.current_char = 1;
        } else {
            self.current_char += 1;
        }
        return next_char;
    }

    pub fn peek(&mut self) -> Option<&char> {
        return self.iter.peek();
    }

    pub fn has_next(&mut self) -> bool {
        return self.iter.peek().is_some();
    }
}

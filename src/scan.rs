//! Scanner for control characters and shell metacharacters. It's presently
//! geared towards Bash and `/bin/sh`.

use std::borrow::Borrow;

#[derive(PartialEq)]
pub(crate) enum Char {
    Bell,
    Backspace,
    Escape,
    FormFeed,
    NewLine,
    CarriageReturn,
    HorizontalTab,
    VerticalTab,
    Backslash,
    SingleQuote,
    ByValue(u8),
    Literal(u8),
    Quoted(u8),
}

impl Char {
    pub fn from<T: Borrow<u8>>(ch: T) -> Self {
        let ch = *ch.borrow();
        use Char::*;
        match ch {
            // Backslash sequences.
            BEL => Bell,
            BS => Backspace,
            ESC => Escape,
            FF => FormFeed,
            LF => NewLine,
            CR => CarriageReturn,
            TAB => HorizontalTab,
            VT => VerticalTab,
            b'\\' => Backslash,
            b'\'' => SingleQuote,

            // ASCII letters, numbers, and "safe" punctuation.
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => Literal(ch),
            b',' | b'.' | b'/' | b'_' | b'-' => Literal(ch),

            // ASCII punctuation which can have significance in the shell.
            b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>' => Quoted(ch),
            b' ' | b'?' | b'[' | b']' | b'{' | b'}' | b'`' => Quoted(ch),
            b'~' | b'!' | b'$' | b'@' | b'+' | b'=' => Quoted(ch),
            b'*' | b'"' | b'%' | b'#' | b':' | b'^' => Quoted(ch),

            // Other control characers.
            0x00..=0x06 | 0x0E..=0x1A | 0x1C..=0x1F | 0x7f => ByValue(ch),

            // High bytes.
            0x80..=0xff => ByValue(ch),
        }
    }

    #[inline]
    pub fn is_literal(&self) -> bool {
        matches!(self, Char::Literal(_))
    }
}

const BEL: u8 = 0x07; // -> \a
const BS: u8 = 0x08; // -> \b
const TAB: u8 = 0x09; // -> \t
const LF: u8 = 0x0A; // -> \n
const VT: u8 = 0x0B; // -> \v
const FF: u8 = 0x0C; // -> \f
const CR: u8 = 0x0D; // -> \r
const ESC: u8 = 0x1B; // -> \e

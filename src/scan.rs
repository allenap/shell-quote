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
    Control(u8),
    Backslash,
    SingleQuote,
    DoubleQuote,
    Delete,
    PrintableInert(u8),
    Printable(u8),
    Extended(u8),
}

impl Char {
    pub fn from<T: Borrow<u8>>(ch: T) -> Self {
        let ch = *ch.borrow();
        use Char::*;
        match ch {
            // ASCII control characters that frequently have dedicated backslash
            // sequences when quoted.
            BEL => Bell,
            BS => Backspace,
            ESC => Escape,
            FF => FormFeed,
            LF => NewLine,
            CR => CarriageReturn,
            TAB => HorizontalTab,
            VT => VerticalTab,

            // ASCII control characters, the rest.
            0x00..=0x06 | 0x0E..=0x1A | 0x1C..=0x1F => Control(ch),

            // ASCII printable characters that can have dedicated backslash
            // sequences when quoted or otherwise need some special treatment.
            b'\\' => Backslash,
            b'\'' => SingleQuote,
            b'\"' => DoubleQuote,
            DEL => Delete,

            // ASCII printable letters, numbers, and "safe" punctuation.
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => PrintableInert(ch),
            b',' | b'.' | b'/' | b'_' | b'-' => PrintableInert(ch),

            // ASCII punctuation which can have significance in the shell.
            b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>' => Printable(ch),
            b' ' | b'?' | b'[' | b']' | b'{' | b'}' | b'`' => Printable(ch),
            b'~' | b'!' | b'$' | b'@' | b'+' | b'=' | b'*' => Printable(ch),
            b'%' | b'#' | b':' | b'^' => Printable(ch),

            // ASCII extended characters, or high bytes.
            0x80..=0xff => Extended(ch),
        }
    }

    #[inline]
    pub fn is_inert(&self) -> bool {
        matches!(self, Char::PrintableInert(_))
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
const DEL: u8 = 0x7F;

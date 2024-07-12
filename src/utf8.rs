#![cfg(any(feature = "bash", feature = "fish"))]

//! Scanner for control codes, shell metacharacters, printable characters, and
//! UTF-8 sequences, i.e. classify each byte in a stream according to where it
//! appears in UTF-8.

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
    Utf8(char),
}

impl Char {
    pub fn from(ch: char) -> Self {
        let ascii: Result<u8, _> = ch.try_into();
        use Char::*;
        match ascii {
            Ok(ascii) => match ascii {
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
                0x00..=0x06 | 0x0E..=0x1A | 0x1C..=0x1F => Control(ascii),

                // ASCII printable characters that can have dedicated backslash
                // sequences when quoted or otherwise need some special treatment.
                b'\\' => Backslash,
                b'\'' => SingleQuote,
                b'\"' => DoubleQuote,
                DEL => Delete,

                // ASCII printable letters, numbers, and "safe" punctuation.
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => PrintableInert(ascii),
                b',' | b'.' | b'/' | b'_' | b'-' => PrintableInert(ascii),

                // ASCII punctuation which can have significance in the shell.
                b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>' => Printable(ascii),
                b' ' | b'?' | b'[' | b']' | b'{' | b'}' | b'`' => Printable(ascii),
                b'~' | b'!' | b'$' | b'@' | b'+' | b'=' | b'*' => Printable(ascii),
                b'%' | b'#' | b':' | b'^' => Printable(ascii),

                // UTF-8 sequences.
                0x80..=0xff => Utf8(ch),
            },
            Err(_) => Utf8(ch),
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

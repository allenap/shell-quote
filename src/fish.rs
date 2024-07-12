#![cfg(feature = "fish")]

use crate::{Quotable, QuoteInto};

/// Quote byte strings for use with fish.
///
/// # ⚠️ Warning
///
/// Prior to version 3.6.2, fish did not correctly handle some Unicode code
/// points encoded as UTF-8. From the [version 3.6.2 release notes][]:
///
/// > fish uses certain Unicode non-characters internally for marking wildcards
/// > and expansions. It incorrectly allowed these markers to be read on command
/// > substitution output, rather than transforming them into a safe internal
/// > representation.
///
/// [version 3.6.2 release notes]:
///   https://github.com/fish-shell/fish-shell/releases/tag/3.6.2
///
/// At present this crate has **no workaround** for this issue. Please use fish
/// 3.6.2 or later.
///
/// # Notes
///
/// The documentation on [quoting][] and [escaping characters][] in fish is
/// confusing at first, especially when coming from a Bourne-like shell, but
/// essentially we have to be able to move and and out of a quoted string
/// context. For example, the escape sequence `\t` for a tab _must_ be outside
/// of quotes, single or double, to be recognised as a tab character by fish:
///
/// ```fish
/// echo 'foo'\t'bar'
/// ```
///
/// This emphasises the importance of using the correct quoting module for the
/// target shell.
///
/// [quoting]: https://fishshell.com/docs/current/language.html#quotes
/// [escaping characters]:
///     https://fishshell.com/docs/current/language.html#escaping-characters
#[derive(Debug, Clone, Copy)]
pub struct Fish;

impl QuoteInto<Vec<u8>> for Fish {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut Vec<u8>) {
        Self::quote_into_vec(s, out);
    }
}

impl QuoteInto<String> for Fish {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut String) {
        Self::quote_into_vec(s, unsafe { out.as_mut_vec() })
    }
}

#[cfg(unix)]
impl QuoteInto<std::ffi::OsString> for Fish {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut std::ffi::OsString) {
        use std::os::unix::ffi::OsStringExt;
        let s = Self::quote_vec(s);
        let s = std::ffi::OsString::from_vec(s);
        out.push(s);
    }
}

#[cfg(feature = "bstr")]
impl QuoteInto<bstr::BString> for Fish {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut bstr::BString) {
        let s = Self::quote_vec(s);
        out.extend(s);
    }
}

impl Fish {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no escaping is necessary.
    /// - An escaped string, like `'foo \'bar'`, `\a'ABC'`
    ///
    /// See [`quote_into_vec`][`Self::quote_into_vec`] for a variant that
    /// extends an existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Fish;
    /// assert_eq!(Fish::quote_vec("foobar"), b"foobar");
    /// assert_eq!(Fish::quote_vec("foo 'bar"), b"foo' \\'bar'");
    /// ```
    pub fn quote_vec<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        match s.into() {
            Quotable::Bytes(bytes) => match bytes::escape_prepare(bytes) {
                bytes::Prepared::Empty => vec![b'\'', b'\''],
                bytes::Prepared::Inert => bytes.into(),
                bytes::Prepared::Escape(esc) => {
                    let mut sout = Vec::new();
                    bytes::escape_chars(esc, &mut sout);
                    sout
                }
            },
            Quotable::Text(text) => match text::escape_prepare(text) {
                text::Prepared::Empty => vec![b'\'', b'\''],
                text::Prepared::Inert => text.into(),
                text::Prepared::Escape(esc) => {
                    let mut sout = Vec::new();
                    text::escape_chars(esc, &mut sout);
                    sout
                }
            },
        }
    }

    /// Quote a string of bytes into an existing `Vec<u8>`.
    ///
    /// See [`quote_vec`][`Self::quote_vec`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Fish;
    /// let mut buf = Vec::with_capacity(128);
    /// Fish::quote_into_vec("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Fish::quote_into_vec("foo 'bar", &mut buf);
    /// assert_eq!(buf, b"foobar foo' \\'bar'");
    /// ```
    ///
    pub fn quote_into_vec<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        match s.into() {
            Quotable::Bytes(bytes) => match bytes::escape_prepare(bytes) {
                bytes::Prepared::Empty => sout.extend(b"''"),
                bytes::Prepared::Inert => sout.extend(bytes),
                bytes::Prepared::Escape(esc) => bytes::escape_chars(esc, sout),
            },
            Quotable::Text(text) => match text::escape_prepare(text) {
                text::Prepared::Empty => sout.extend(b"''"),
                text::Prepared::Inert => sout.extend(text.as_bytes()),
                text::Prepared::Escape(esc) => text::escape_chars(esc, sout),
            },
        }
    }
}

// ----------------------------------------------------------------------------

mod bytes {
    use super::u8_to_hex_escape_uppercase_x;
    use crate::ascii::Char;

    pub enum Prepared {
        Empty,
        Inert,
        Escape(Vec<Char>),
    }

    pub fn escape_prepare(sin: &[u8]) -> Prepared {
        let esc: Vec<_> = sin.iter().map(Char::from).collect();
        // An optimisation: if the string is not empty and contains only "safe"
        // characters we can avoid further work.
        if esc.is_empty() {
            Prepared::Empty
        } else if esc.iter().all(Char::is_inert) {
            Prepared::Inert
        } else {
            Prepared::Escape(esc)
        }
    }

    pub fn escape_chars(esc: Vec<Char>, sout: &mut Vec<u8>) {
        #[derive(PartialEq)]
        enum QuoteStyle {
            Inside,
            Outside,
            Whatever,
        }
        use QuoteStyle::*;

        let mut inside_quotes_now = false;
        let mut push_literal = |style: QuoteStyle, literal: &[u8]| {
            match (inside_quotes_now, style) {
                (true, Outside) => {
                    sout.push(b'\'');
                    inside_quotes_now = false;
                }
                (false, Inside) => {
                    sout.push(b'\'');
                    inside_quotes_now = true;
                }
                _ => (),
            }
            sout.extend(literal);
        };
        for mode in esc {
            use Char::*;
            match mode {
                Bell => push_literal(Outside, b"\\a"),
                Backspace => push_literal(Outside, b"\\b"),
                Escape => push_literal(Outside, b"\\e"),
                FormFeed => push_literal(Outside, b"\\f"),
                NewLine => push_literal(Outside, b"\\n"),
                CarriageReturn => push_literal(Outside, b"\\r"),
                HorizontalTab => push_literal(Outside, b"\\t"),
                VerticalTab => push_literal(Outside, b"\\v"),
                Control(ch) => push_literal(Outside, &u8_to_hex_escape_uppercase_x(ch)),
                Backslash => push_literal(Whatever, b"\\\\"),
                SingleQuote => push_literal(Whatever, b"\\'"),
                DoubleQuote => push_literal(Inside, b"\""),
                Delete => push_literal(Outside, b"\\X7F"),
                PrintableInert(ch) => push_literal(Whatever, &ch.to_le_bytes()),
                Printable(ch) => push_literal(Inside, &ch.to_le_bytes()),
                Extended(ch) => push_literal(Outside, &u8_to_hex_escape_uppercase_x(ch)),
            }
        }
        if inside_quotes_now {
            sout.push(b'\'');
        }
    }
}

// ----------------------------------------------------------------------------

mod text {
    use super::u8_to_hex_escape_uppercase_x;
    use crate::utf8::Char;

    pub enum Prepared {
        Empty,
        Inert,
        Escape(Vec<Char>),
    }

    pub fn escape_prepare(sin: &str) -> Prepared {
        let esc: Vec<_> = sin.chars().map(Char::from).collect();
        // An optimisation: if the string is not empty and contains only "safe"
        // characters we can avoid further work.
        if esc.is_empty() {
            Prepared::Empty
        } else if esc.iter().all(Char::is_inert) {
            Prepared::Inert
        } else {
            Prepared::Escape(esc)
        }
    }

    pub fn escape_chars(esc: Vec<Char>, sout: &mut Vec<u8>) {
        #[derive(PartialEq)]
        enum QuoteStyle {
            Inside,
            Outside,
            Whatever,
        }
        use QuoteStyle::*;

        let mut inside_quotes_now = false;
        let mut push_literal = |style: QuoteStyle, literal: &[u8]| {
            match (inside_quotes_now, style) {
                (true, Outside) => {
                    sout.push(b'\'');
                    inside_quotes_now = false;
                }
                (false, Inside) => {
                    sout.push(b'\'');
                    inside_quotes_now = true;
                }
                _ => (),
            }
            sout.extend(literal);
        };
        let buf = &mut [0u8; 4];
        for mode in esc {
            use Char::*;
            match mode {
                Bell => push_literal(Outside, b"\\a"),
                Backspace => push_literal(Outside, b"\\b"),
                Escape => push_literal(Outside, b"\\e"),
                FormFeed => push_literal(Outside, b"\\f"),
                NewLine => push_literal(Outside, b"\\n"),
                CarriageReturn => push_literal(Outside, b"\\r"),
                HorizontalTab => push_literal(Outside, b"\\t"),
                VerticalTab => push_literal(Outside, b"\\v"),
                Control(ch) => push_literal(Outside, &u8_to_hex_escape_uppercase_x(ch)),
                Backslash => push_literal(Whatever, b"\\\\"),
                SingleQuote => push_literal(Whatever, b"\\'"),
                DoubleQuote => push_literal(Inside, b"\""),
                Delete => push_literal(Outside, b"\\X7F"),
                PrintableInert(ch) => push_literal(Whatever, &ch.to_le_bytes()),
                Printable(ch) => push_literal(Inside, &ch.to_le_bytes()),
                Utf8(char) => push_literal(Inside, char.encode_utf8(buf).as_bytes()),
            }
        }
        if inside_quotes_now {
            sout.push(b'\'');
        }
    }
}

// ----------------------------------------------------------------------------

/// Escape a byte as a 4-byte hex escape sequence _with uppercase "X"_.
///
/// The `\\XHH` format (backslash, a literal "X", two hex characters) is
/// understood by fish. The `\\xHH` format is _also_ understood, but until fish
/// 3.6.0 it had a weirdness. From the [release notes][]:
///
/// > The `\\x` and `\\X` escape syntax is now equivalent. `\\xAB` previously
/// > behaved the same as `\\XAB`, except that it would error if the value “AB”
/// > was larger than “7f” (127 in decimal, the highest ASCII value).
///
/// [release notes]: https://github.com/fish-shell/fish-shell/releases/tag/3.6.0
///
#[inline]
fn u8_to_hex_escape_uppercase_x(ch: u8) -> [u8; 4] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        b'\\',
        b'X',
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

#[cfg(test)]
#[test]
fn test_u8_to_hex_escape_uppercase_x() {
    for ch in u8::MIN..=u8::MAX {
        let expected = format!("\\X{ch:02X}");
        let observed = u8_to_hex_escape_uppercase_x(ch);
        let observed = std::str::from_utf8(&observed).unwrap();
        assert_eq!(observed, &expected);
    }
}

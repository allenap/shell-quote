#![cfg(feature = "bash")]

use crate::{ascii::Char, util::u8_to_hex_escape, Quotable, QuoteInto};

/// Quote byte strings for use with Bash, the GNU Bourne-Again Shell.
///
/// # Compatibility
///
/// Quoted/escaped strings produced by [`Bash`] work in both Bash and Z Shell.
///
/// # ⚠️ Warning
///
/// It is _possible_ to encode NUL in a Bash string, but Bash appears to then
/// truncate the rest of the string after that point **or** sometimes it filters
/// the NUL out. It's not yet clear to me when/why each behaviour is chosen.
///
/// If you're quoting UTF-8 content this may not be a problem since there is
/// only one code point – the null character itself – that will ever produce a
/// NUL byte. To avoid this problem entirely, consider using [Modified
/// UTF-8][modified-utf-8] so that the NUL byte can never appear in a valid byte
/// stream.
///
/// [modified-utf-8]: https://en.wikipedia.org/wiki/UTF-8#Modified_UTF-8
///
/// # Notes
///
/// From bash(1):
///
///   Words of the form $'string' are treated specially. The word expands to
///   string, with backslash-escaped characters replaced as specified by the
///   ANSI C standard. Backslash escape sequences, if present, are decoded as
///   follows:
///
///   ```text
///   \a     alert (bell)
///   \b     backspace
///   \e     an escape character
///   \f     form feed
///   \n     new line
///   \r     carriage return
///   \t     horizontal tab
///   \v     vertical tab
///   \\     backslash
///   \'     single quote
///   \nnn   the eight-bit character whose value is the
///          octal value nnn (one to three digits)
///   \xHH   the eight-bit character whose value is the
///          hexadecimal value HH (one or two hex digits)
///   \cx    a control-x character
///   ```
///
/// Bash allows, in newer versions, for non-ASCII Unicode characters with
/// `\uHHHH` and `\UXXXXXXXX` syntax inside these [ANSI C quoted
/// strings][ansi-c-quoting], but we avoid this and work only with bytes. Part
/// of the problem is that it's not clear how Bash then works with these
/// strings. Does it encode these characters into bytes according to the user's
/// current locale? Are strings in Bash now natively Unicode?
///
/// For now it's up to the caller to figure out encoding. A significant use case
/// for this code is to quote filenames into scripts, and on *nix variants I
/// understand that filenames are essentially arrays of bytes, even if the OS
/// adds some normalisation and case-insensitivity on top.
///
/// [ansi-c-quoting]:
///     https://www.gnu.org/software/bash/manual/html_node/ANSI_002dC-Quoting.html
///
#[derive(Debug, Clone, Copy)]
pub struct Bash;

// ----------------------------------------------------------------------------

impl QuoteInto<Vec<u8>> for Bash {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut Vec<u8>) {
        Self::quote_into_vec(s, out);
    }
}

impl QuoteInto<String> for Bash {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut String) {
        Self::quote_into_vec(s, unsafe { out.as_mut_vec() })
    }
}

#[cfg(unix)]
impl QuoteInto<std::ffi::OsString> for Bash {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut std::ffi::OsString) {
        use std::os::unix::ffi::OsStringExt;
        let s = Self::quote_vec(s);
        let s = std::ffi::OsString::from_vec(s);
        out.push(s);
    }
}

#[cfg(feature = "bstr")]
impl QuoteInto<bstr::BString> for Bash {
    fn quote_into<'q, S: ?Sized + Into<Quotable<'q>>>(s: S, out: &mut bstr::BString) {
        let s = Self::quote_vec(s);
        out.extend(s);
    }
}

// ----------------------------------------------------------------------------

impl Bash {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no escaping is necessary.
    /// - An [ANSI-C escaped string][ansi-c-quoting], like `$'foo\nbar'`.
    ///
    /// See [`quote_into`](#method.quote_into) for a variant that extends an
    /// existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Bash;
    /// assert_eq!(Bash::quote_vec("foobar"), b"foobar");
    /// assert_eq!(Bash::quote_vec("foo bar"), b"$'foo bar'");
    /// ```
    ///
    /// [ansi-c-quoting]:
    ///     https://www.gnu.org/software/bash/manual/html_node/ANSI_002dC-Quoting.html
    ///
    pub fn quote_vec<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => vec![b'\'', b'\''],
            Prepared::Inert => sin.bytes.into(),
            Prepared::Escape(esc) => {
                // This may be a pointless optimisation, but calculate the
                // memory needed to avoid reallocations as we construct the
                // output. Since we'll generate a $'…' string, add 3 bytes.
                let size: usize = esc.iter().map(escape_size).sum();
                let mut sout = Vec::with_capacity(size + 3);
                escape_chars(esc, &mut sout); // Do the work.
                sout
            }
        }
    }

    /// Quote a string of bytes into an existing `Vec<u8>`.
    ///
    /// See [`quote`](#method.quote) for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Bash;
    /// let mut buf = Vec::with_capacity(128);
    /// Bash::quote_into_vec("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Bash::quote_into_vec("foo bar", &mut buf);
    /// assert_eq!(buf, b"foobar $'foo bar'");
    /// ```
    ///
    pub fn quote_into_vec<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => sout.extend(b"''"),
            Prepared::Inert => sout.extend(sin.bytes),
            Prepared::Escape(esc) => {
                // This may be a pointless optimisation, but calculate the
                // memory needed to avoid reallocations as we construct the
                // output. Since we'll generate a $'…' string, add 3 bytes.
                let size: usize = esc.iter().map(escape_size).sum();
                sout.reserve(size + 3);
                let cap = sout.capacity();
                escape_chars(esc, sout); // Do the work.
                debug_assert_eq!(cap, sout.capacity()); // No reallocations.
            }
        }
    }
}

// ----------------------------------------------------------------------------

enum Prepared {
    Empty,
    Inert,
    Escape(Vec<Char>),
}

fn escape_prepare(sin: &[u8]) -> Prepared {
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

fn escape_chars(esc: Vec<Char>, sout: &mut Vec<u8>) {
    // Push a Bash-style $'...' quoted string into `sout`.
    sout.extend(b"$'");
    for mode in esc {
        use Char::*;
        match mode {
            Bell => sout.extend(b"\\a"),
            Backspace => sout.extend(b"\\b"),
            Escape => sout.extend(b"\\e"),
            FormFeed => sout.extend(b"\\f"),
            NewLine => sout.extend(b"\\n"),
            CarriageReturn => sout.extend(b"\\r"),
            HorizontalTab => sout.extend(b"\\t"),
            VerticalTab => sout.extend(b"\\v"),
            Control(ch) => sout.extend(&u8_to_hex_escape(ch)),
            Backslash => sout.extend(b"\\\\"),
            SingleQuote => sout.extend(b"\\'"),
            DoubleQuote => sout.extend(b"\""),
            Delete => sout.extend(b"\\x7F"),
            PrintableInert(ch) => sout.push(ch),
            Printable(ch) => sout.push(ch),
            Extended(ch) => sout.extend(&u8_to_hex_escape(ch)),
        }
    }
    sout.push(b'\'');
}

fn escape_size(char: &Char) -> usize {
    use Char::*;
    match char {
        Bell => 2,
        Backspace => 2,
        Escape => 2,
        FormFeed => 2,
        NewLine => 2,
        CarriageReturn => 2,
        HorizontalTab => 2,
        VerticalTab => 2,
        Control(_) => 4,
        Backslash => 2,
        SingleQuote => 2,
        DoubleQuote => 1,
        Delete => 4,
        PrintableInert(_) => 1,
        Printable(_) => 1,
        Extended(_) => 4,
    }
}

use crate::{ascii::Char, quoter::QuoterSealed, Quoter};

/// Quote byte strings for use with `/bin/sh`.
///
/// # Notes
///
/// The following escapes seem to be "okay":
///
/// ```text
/// \a     alert (bell)
/// \b     backspace
/// \f     form feed
/// \n     new line
/// \r     carriage return
/// \t     horizontal tab
/// \v     vertical tab
/// \\     backslash
/// \nnn   the eight-bit character whose value is the octal value nnn
/// ```
///
/// I wasn't able to find any definitive statement of exactly how Bourne Shell
/// strings should be quoted, mainly because "Bourne Shell" or `/bin/sh` can
/// refer to many different pieces of software: Bash has a Bourne Shell mode,
/// `/bin/sh` on Ubuntu is actually Dash, and on macOS 12.3 (and later, and
/// possibly earlier) all bets are off:
///
/// > `sh` is a POSIX-compliant command interpreter (shell). It is implemented
/// > by re-execing as either `bash(1)`, `dash(1)`, or `zsh(1)` as determined by
/// > the symbolic link located at `/private/var/select/sh`. If
/// > `/private/var/select/sh` does not exist or does not point to a valid
/// > shell, `sh` will use one of the supported shells.
///
/// ⚠️ In practice, however, bytes between 0x80 and 0xff inclusive **cannot** be
/// escaped with `\nnn` notation. The shell simply ignores these escapes and
/// treats `\nnn` as a literal string of 4 characters. Hence, in this module,
/// these bytes are reproduced as-is within the quoted string output, with no
/// special escaping.
///
/// The code in this module sticks to escape sequences that I consider
/// "standard" by a heuristic known only to me. It operates byte by byte, making
/// no special allowances for multi-byte character sets. In other words, it's up
/// to the caller to figure out encoding for non-ASCII characters. A significant
/// use case for this code is to quote filenames into scripts, and on *nix
/// variants I understand that filenames are essentially arrays of bytes, even
/// if the OS adds some normalisation and case-insensitivity on top.
///
/// If you have some expertise in this area I would love to hear from you.
///
#[derive(Debug, Clone, Copy)]
pub struct Sh;

impl Quoter for Sh {}

/// Expose [`Quoter`] implementation as default impl too, for convenience.
impl QuoterSealed for Sh {
    fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8> {
        Self::quote(s)
    }
    fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>) {
        Self::quote_into(s, sout)
    }
}

impl Sh {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no quoting is necessary.
    /// - A quoted string containing ANSI-C-like escapes, like `'foo\nbar'`.
    ///
    /// See [`quote_into`](#method.quote_into) for a variant that extends an
    /// existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::{Sh, Quoter};
    /// assert_eq!(Sh::quote("foobar"), b"foobar");
    /// assert_eq!(Sh::quote("foo bar"), b"'foo bar'");
    /// ```
    ///
    pub fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8> {
        let sin = s.as_ref();
        if let Some(esc) = escape_prepare(sin) {
            // This may be a pointless optimisation, but calculate the memory
            // needed to avoid reallocations as we construct the output. Since
            // we know we're going to use single quotes, we also add 2 bytes.
            let size: usize = esc.iter().map(escape_size).sum();
            let mut sout = Vec::with_capacity(size + 2);
            escape_chars(esc, &mut sout); // Do the work.
            sout
        } else {
            sin.into()
        }
    }

    /// Quote a string of bytes into an existing `Vec<u8>`.
    ///
    /// See [`quote`](#method.quote) for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::{Sh, Quoter};
    /// let mut buf = Vec::with_capacity(128);
    /// Sh::quote_into("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Sh::quote_into("foo bar", &mut buf);
    /// assert_eq!(buf, b"foobar 'foo bar'");
    /// ```
    ///
    pub fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>) {
        let sin = s.as_ref();
        if let Some(esc) = escape_prepare(sin) {
            // This may be a pointless optimisation, but calculate the memory
            // needed to avoid reallocations as we construct the output. Since
            // we know we're going to use single quotes, we also add 2 bytes.
            let size: usize = esc.iter().map(escape_size).sum();
            sout.reserve(size + 2);
            escape_chars(esc, sout); // Do the work.
        } else {
            sout.extend(sin);
        }
    }
}

// ----------------------------------------------------------------------------

fn escape_prepare(sin: &[u8]) -> Option<Vec<Char>> {
    let esc: Vec<_> = sin.iter().map(Char::from).collect();
    // An optimisation: if the string is not empty and contains only "safe"
    // characters we can avoid further work.
    if esc.is_empty() {
        Some(esc)
    } else if esc.iter().all(Char::is_inert) {
        None
    } else {
        Some(esc)
    }
}

fn escape_chars(esc: Vec<Char>, sout: &mut Vec<u8>) {
    // Push a Bourne-style '...' escaped string into `sout`.
    sout.extend(b"'");
    for mode in esc {
        use Char::*;
        match mode {
            Bell => sout.extend(b"\\a"),
            Backspace => sout.extend(b"\\b"),
            Escape => sout.extend(b"\\033"),
            FormFeed => sout.extend(b"\\f"),
            NewLine => sout.extend(b"\\n"),
            CarriageReturn => sout.extend(b"\\r"),
            HorizontalTab => sout.extend(b"\\t"),
            VerticalTab => sout.extend(b"\\v"),
            Control(ch) => sout.extend(format!("\\{:03o}", ch).bytes()),
            Backslash => sout.extend(b"\\\\"),
            SingleQuote => sout.extend(b"\\047"),
            DoubleQuote => sout.extend(b"\""),
            Delete => sout.push(0x7F),
            PrintableInert(ch) => sout.push(ch),
            Printable(ch) => sout.push(ch),
            Extended(ch) => sout.push(ch),
        }
    }
    sout.push(b'\'');
}

fn escape_size(char: &Char) -> usize {
    use Char::*;
    match char {
        Bell => 2,
        Backspace => 2,
        Escape => 4,
        FormFeed => 2,
        NewLine => 2,
        CarriageReturn => 2,
        HorizontalTab => 2,
        VerticalTab => 2,
        Control(_) => 4,
        Backslash => 2,
        SingleQuote => 4,
        DoubleQuote => 1,
        Delete => 4,
        PrintableInert(_) => 1,
        Printable(_) => 1,
        Extended(_) => 4,
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::os::unix::prelude::OsStringExt;
    use std::{ffi::OsString, os::unix::ffi::OsStrExt};

    use crate::find_bins;

    use super::Sh;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            Sh::quote("abcdefghijklmnopqrstuvwxyz"),
            b"abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            Sh::quote("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(Sh::quote("0123456789"), b"0123456789");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(Sh::quote("-_=/,.+"), b"'-_=/,.+'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Sh::quote(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Sh::quote(r#"woo'wah""#), br#"'woo\047wah"'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Sh::quote("\x07"), b"'\\a'");
        assert_eq!(Sh::quote("\x00"), b"'\\000'");
        assert_eq!(Sh::quote("\x06"), b"'\\006'");
        assert_eq!(Sh::quote("\x7F"), b"'\x7F'");
        assert_eq!(Sh::quote("\x1B"), b"'\\033'");
    }

    #[test]
    fn test_quote_into_plain() {
        let mut buffer = Vec::new();
        Sh::quote_into("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_quote_into_with_escapes() {
        let mut buffer = Vec::new();
        Sh::quote_into("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"'-_=/,.+'");
    }

    #[test]
    fn test_roundtrip() {
        use std::process::Command;
        // In Bash it doesn't seem possible to roundtrip NUL, but in the Bourne
        // shell, or whatever is masquerading as `sh`, it seems to be fine.
        let string: OsString = OsString::from_vec((u8::MIN..=u8::MAX).collect());
        let mut script = b"echo ".to_vec();
        Sh::quote_into(string.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        for bin in find_bins("sh") {
            let output = Command::new(bin).arg("-c").arg(&script).output().unwrap();
            let mut result = output.stdout;
            result.resize(result.len() - 1, 0); // Remove trailing newline.
            let result = OsString::from_vec(result);
            assert_eq!(result, string);
        }
    }
}

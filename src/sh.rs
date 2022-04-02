//! Quote strings for use with `/bin/sh`.

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use crate::scan::Char;

/// Escape a string of *bytes* into a new `Vec<u8>`.
///
/// This will return one of the following:
/// - The string as-is, if no escaping is necessary.
/// - An ANSI-C-like escaped string, like `'foo bar'`.
///
/// See [`escape_into`][] for a variant that extends an existing `Vec` instead
/// of allocating a new one.
///
/// [`escape_into`]: ./fn.escape_into.html
///
/// # Examples
///
/// ```
/// # use shell_quote::sh;
/// assert_eq!(sh::escape("foobar"), b"foobar");
/// assert_eq!(sh::escape("foo bar"), b"'foo bar'");
/// ```
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
/// strings should be escaped, mainly because "Bourne Shell" or `/bin/sh` can
/// refer to many different pieces of software: Bash has a Bourne Shell mode,
/// `/bin/sh` on Ubuntu is actually Dash, and on macOS 12.3 (and later, and
/// possibly earlier) all bets are off:
///
/// > `sh` is a POSIX-compliant command interpreter (shell). It is implemented
/// > by re-execing as either `bash`(1), `dash`(1), or `zsh`(1) as determined by
/// > the symbolic link located at `/private/var/select/sh`. If
/// > `/private/var/select/sh` does not exist or does not point to a valid
/// > shell, `sh` will use one of the supported shells.
///
/// The code in this module sticks to escape sequences that I consider
/// "standard" by a heuristic known only to me. It operates byte by byte, making
/// no special allowances for multi-byte character sets. In other words, it's up
/// to the caller to figure out encoding for non-ASCII characters. A significant
/// use case for this code is to escape filenames into scripts, and on *nix
/// variants I understand that filenames are essentially arrays of bytes, even
/// if the OS adds some normalisation and case-insensitivity on top.
///
/// If you have some expertise in this area I would love to hear from you.
///
/// The argument passed into `escape` is `Into<OsString>`, so you can pass in
/// regular Rust strings, `PathBuf`, and so on. For a regular Rust string it
/// will be quoted byte for byte
///
pub fn escape<T: Into<OsString>>(s: T) -> Vec<u8> {
    let sin = s.into().into_vec();
    if let Some(esc) = escape_prepare(&sin) {
        // Maybe pointless optimisation, but here we calculate the memory we need to
        // avoid reallocations as we construct the output string. Since we now know
        // we're going to use single quotes, we also add 2 bytes.
        let size: usize = esc.iter().map(escape_size).sum();
        let mut sout = Vec::with_capacity(size + 2);
        escape_chars(esc, &mut sout); // Do the work.
        sout
    } else {
        sin
    }
}

/// Escape a string of *bytes* into an existing `Vec<u8>`.
///
/// See [`escape`][] for more details.
///
/// [`escape`]: ./fn.escape.html
///
/// # Examples
///
/// ```
/// # use shell_quote::sh;
/// let mut buf = Vec::with_capacity(128);
/// sh::escape_into("foobar", &mut buf);
/// buf.push(b' ');  // Add a space.
/// sh::escape_into("foo bar", &mut buf);
/// assert_eq!(buf, b"foobar 'foo bar'");
/// ```
///
pub fn escape_into<T: Into<OsString>>(s: T, sout: &mut Vec<u8>) {
    let sin = s.into().into_vec();
    if let Some(esc) = escape_prepare(&sin) {
        // Maybe pointless optimisation, but here we calculate the memory we
        // need to avoid reallocations as we construct the output string. Since
        // we now know we're going to use single quotes, we also add 2 bytes.
        let size: usize = esc.iter().map(escape_size).sum();
        sout.reserve(size + 2);
        escape_chars(esc, sout); // Do the work.
    } else {
        sout.extend(sin);
    }
}

fn escape_prepare(sin: &[u8]) -> Option<Vec<Char>> {
    let esc: Vec<_> = sin.iter().map(Char::from).collect();
    // An optimisation: if the string only contains "safe" characters we can
    // avoid further work.
    if esc.iter().all(Char::is_literal) {
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
            Backslash => sout.extend(b"\\\\"),
            SingleQuote => sout.extend(b"\\047"),
            ByValue(ch) => sout.extend(format!("\\{:03o}", ch).bytes()),
            Literal(ch) => sout.push(ch),
            Quoted(ch) => sout.push(ch),
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
        Backslash => 2,
        SingleQuote => 4,
        ByValue(_) => 4,
        Literal(_) => 1,
        Quoted(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::escape;
    use super::escape_into;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            escape("abcdefghijklmnopqrstuvwxyz"),
            b"abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            escape("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(escape("0123456789"), b"0123456789");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(escape("-_=/,.+"), b"'-_=/,.+'");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(escape(r#"woo'wah""#), br#"'woo\047wah"'"#);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_control_characters() {
        assert_eq!(escape(&"\x07"), b"'\\a'");
        assert_eq!(escape(&"\x00"), b"'\\000'");
        assert_eq!(escape(&"\x06"), b"'\\006'");
        assert_eq!(escape(&"\x7F"), b"'\\177'");
        assert_eq!(escape(&"\x7F"), b"'\\177'");
        assert_eq!(escape(&"\x1B"), b"'\\033'");
    }

    #[test]
    fn test_escape_into_plain() {
        let mut buffer = Vec::new();
        escape_into("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_escape_into_with_escapes() {
        let mut buffer = Vec::new();
        escape_into("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"'-_=/,.+'");
    }
}

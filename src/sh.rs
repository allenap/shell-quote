#![cfg(feature = "sh")]

use crate::{ascii::Char, quoter::QuoterSealed, Quotable, Quoter};

/// Quote byte strings for use with `/bin/sh`.
///
/// # Compatibility
///
/// Quoted/escaped strings produced by [`Sh`] also work in Bash, Dash, and Z
/// Shell.
///
/// The quoted/escaped strings it produces are different to those coming from
/// [`Bash`][`crate::Bash`] or its alias [`Zsh`][`crate::Zsh`]. Those strings
/// won't work in a pure `/bin/sh` shell like Dash, but they are better for
/// humans to read, to copy and paste. For example, [`Sh`] does not (and cannot)
/// escape control characters, but characters like `BEL` and `TAB` (and others)
/// are represented by `\\a` and `\\t` respectively by [`Bash`][`crate::Bash`].
///
/// # Notes
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
/// However, [dash](https://en.wikipedia.org/wiki/Almquist_shell#dash) appears
/// to be the de facto `/bin/sh` these days, having been formally adopted in
/// Ubuntu and Debian, and also available as `/bin/dash` on macOS.
///
/// From dash(1):
///
/// > ## Quoting
/// >
/// >   Quoting is used to remove the special meaning of certain characters or
/// >   words to the shell, such as operators, whitespace, or keywords.  There
/// >   are three types of quoting: matched single quotes, matched double
/// >   quotes, and backslash.
/// >
/// > ## Backslash
/// >
/// >   A backslash preserves the literal meaning of the following character,
/// >   with the exception of ⟨newline⟩.  A backslash preceding a ⟨newline⟩ is
/// >   treated as a line continuation.
/// >
/// > ## Single Quotes
/// >
/// >   Enclosing characters in single quotes preserves the literal meaning of
/// >   all the characters (except single quotes, making it impossible to put
/// >   single-quotes in a single-quoted string).
/// >
/// > ## Double Quotes
/// >
/// >   Enclosing characters within double quotes preserves the literal meaning
/// >   of all characters except dollarsign ($), backquote (`), and backslash
/// >   (\).  The backslash inside double quotes is historically weird, and
/// >   serves to quote only the following characters:
/// >
/// >   ```text
/// >   $ ` " \ <newline>.
/// >   ```
/// >
/// >   Otherwise it remains literal.
///
/// The code in this module operates byte by byte, making no special allowances
/// for multi-byte character sets. In other words, it's up to the caller to
/// figure out encoding for non-ASCII characters. A significant use case for
/// this code is to quote filenames into scripts, and on *nix variants I
/// understand that filenames are essentially arrays of bytes, even if the OS
/// adds some normalisation and case-insensitivity on top.
///
/// If you have some expertise in this area I would love to hear from you.
///
#[derive(Debug, Clone, Copy)]
pub struct Sh;

impl Quoter for Sh {}

/// Expose [`Quoter`] implementation as default impl too, for convenience.
impl QuoterSealed for Sh {
    fn quote<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        Self::quote(s)
    }
    fn quote_into<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        Self::quote_into(s, sout)
    }
}

impl Sh {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no quoting is necessary.
    /// - A string containing single-quoted sections, like `foo' bar'`.
    ///
    /// See [`quote_into`](#method.quote_into) for a variant that extends an
    /// existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::{Sh, Quoter};
    /// assert_eq!(Sh::quote("foobar"), b"foobar");
    /// assert_eq!(Sh::quote("foo bar"), b"foo' bar'");
    /// ```
    ///
    pub fn quote<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => vec![b'\'', b'\''],
            Prepared::Inert => sin.bytes.into(),
            Prepared::Escape(esc) => {
                // This may be a pointless optimisation, but calculate the
                // memory needed to avoid reallocations as we construct the
                // output. Since we'll generate a '…' string, add 2 bytes. Each
                // single quote can consume an extra 1 to 3 bytes, so we assume
                // the worse.
                let quotes: usize = esc
                    .iter()
                    .filter(|char| **char == Char::SingleQuote)
                    .count();

                let size: usize = esc.len() + 2 + (quotes * 3);
                let mut sout = Vec::with_capacity(size);
                let cap = sout.capacity();
                escape_chars(esc, &mut sout); // Do the work.
                debug_assert_eq!(cap, sout.capacity()); // No reallocations.
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
    /// # use shell_quote::{Sh, Quoter};
    /// let mut buf = Vec::with_capacity(128);
    /// Sh::quote_into("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Sh::quote_into("foo bar", &mut buf);
    /// assert_eq!(buf, b"foobar foo' bar'");
    /// ```
    ///
    pub fn quote_into<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => sout.extend(b"''"),
            Prepared::Inert => sout.extend(sin.bytes),
            Prepared::Escape(esc) => {
                // This may be a pointless optimisation, but calculate the
                // memory needed to avoid reallocations as we construct the
                // output. Since we'll generate a '…' string, add 2 bytes. Each
                // single quote can consume an extra 1 to 3 bytes, so we assume
                // the worse.
                let quotes: usize = esc
                    .iter()
                    .filter(|char| **char == Char::SingleQuote)
                    .count();

                let size: usize = esc.len() + 2 + (quotes * 3);
                sout.reserve(size);
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
    let mut inside_quotes_now = false;
    for mode in esc {
        use Char::*;
        match mode {
            PrintableInert(ch) => sout.push(ch),
            Control(ch) | Printable(ch) | Extended(ch) => {
                if inside_quotes_now {
                    sout.push(ch);
                } else {
                    sout.push(b'\'');
                    inside_quotes_now = true;
                    sout.push(ch);
                }
            }
            SingleQuote => {
                if inside_quotes_now {
                    sout.extend(b"'\\'");
                    inside_quotes_now = false;
                } else {
                    sout.extend(b"\\'");
                }
            }
            ch => {
                if inside_quotes_now {
                    sout.push(ch.code());
                } else {
                    sout.push(b'\'');
                    inside_quotes_now = true;
                    sout.push(ch.code());
                }
            }
        }
    }
    if inside_quotes_now {
        sout.push(b'\'');
    }
}

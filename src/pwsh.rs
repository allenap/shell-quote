#![cfg(feature = "pwsh")]

use crate::{Quotable, QuoteInto};

/// Quote byte strings for use with [PowerShell][], `pwsh`.
///
/// [PowerShell]: https://en.wikipedia.org/wiki/PowerShell
///
/// In the [Windows PowerShell Language Specification 3.0][] [§ 2.3.5.2 String
/// literals][], several quoting styles are described. This module renders
/// `verbatim-string-literal` only.
///
/// [Windows PowerShell Language Specification 3.0]:
///     https://learn.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-01
/// [§ 2.3.5.2 String literals]:
///     https://learn.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-02?view=powershell-7.5#2352-string-literals
///
/// # ⚠️ Warning
///
/// PowerShell strings must be Unicode according to the specification. This
/// means one cannot include arbitrary binary data in a PowerShell string.
/// Hence, for now, there is no [`QuoteInto<String>`] implementation for
/// [`Pwsh`].
///
/// If you're only using Unicode, a workaround is to instead quote into a
/// [`Vec<u8>`] and convert that into a string using [`String::from_utf8`]. The
/// key difference is that `from_utf8` returns a [`Result`] which the caller
/// must deal with.
///
#[derive(Debug, Clone, Copy)]
pub struct Pwsh;

impl QuoteInto<Vec<u8>> for Pwsh {
    fn quote_into<'q, S: Into<Quotable<'q>>>(s: S, out: &mut Vec<u8>) {
        Self::quote_into_vec(s, out);
    }
}

#[cfg(unix)]
impl QuoteInto<std::ffi::OsString> for Pwsh {
    fn quote_into<'q, S: Into<Quotable<'q>>>(s: S, out: &mut std::ffi::OsString) {
        use std::os::unix::ffi::OsStringExt;
        let s = Self::quote_vec(s);
        let s = std::ffi::OsString::from_vec(s);
        out.push(s);
    }
}

#[cfg(feature = "bstr")]
impl QuoteInto<bstr::BString> for Pwsh {
    fn quote_into<'q, S: Into<Quotable<'q>>>(s: S, out: &mut bstr::BString) {
        let s = Self::quote_vec(s);
        out.extend(s);
    }
}

impl Pwsh {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no quoting is necessary.
    /// - A string containing single-quoted sections, like `'foo bar'`.
    ///
    /// See [`quote_into_vec`][`Self::quote_into_vec`] for a variant that
    /// extends an existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Pwsh;
    /// assert_eq!(Pwsh::quote_vec("foobar"), b"'foobar'");
    /// assert_eq!(Pwsh::quote_vec("foo bar"), b"'foo bar'");
    /// ```
    ///
    pub fn quote_vec<'a, S: Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        let mut sout = Vec::new();
        Self::quote_into_vec(s, &mut sout);
        sout
    }

    /// Quote a string of bytes into an existing `Vec<u8>`.
    ///
    /// See [`quote_vec`][`Self::quote_vec`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::Pwsh;
    /// let mut buf = Vec::with_capacity(128);
    /// Pwsh::quote_into_vec("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Pwsh::quote_into_vec("foo bar", &mut buf);
    /// assert_eq!(buf, b"'foobar' 'foo bar'");  // Invalid PowerShell; see note below.
    /// ```
    ///
    /// ⚠️ Note that *when pushing multiple items* no attempt is made to create
    /// syntactically valid PowerShell. In particular, string literals are
    /// concatenated with the `+` operator, not by juxtaposition. It's up to the
    /// caller to add the necessary `+` operators.
    ///
    pub fn quote_into_vec<'a, S: Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        let bytes = match s.into() {
            Quotable::Bytes(bytes) => bytes,
            Quotable::Text(s) => s.as_bytes(),
        };
        sout.push(b'\'');
        let mut last: [u8; 2] = [0x00, 0x00]; // Used to track UTF-8 sequences.
        let sout = bytes.iter().fold(sout, |sout, ch| {
            match *ch {
                // Escape multi-byte single quotes by doubling them.
                0xE2 if last == [0x00, 0x00] => last = [0x00, 0xE2], // Start of a UTF-8 sequence.
                0x80 if last == [0x00, 0xE2] => last = [0xE2, 0x80], // Continuation byte of a UTF-8 sequence.
                0x98 if last == [0xE2, 0x80] => {
                    sout.extend("\u{2018}\u{2018}".as_bytes()); // Left single quotation mark (U+2018).
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }
                0x99 if last == [0xE2, 0x80] => {
                    sout.extend("\u{2019}\u{2019}".as_bytes()); // Right single quotation mark (U+2019).
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }
                0x9A if last == [0xE2, 0x80] => {
                    sout.extend("\u{201A}\u{201A}".as_bytes()); // Single low-9 quotation mark (U+201A).
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }
                0x9B if last == [0xE2, 0x80] => {
                    sout.extend("\u{201B}\u{201B}".as_bytes()); // Single high-reversed-9 quotation mark (U+201B).
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }

                // We're in a UTF-8 sequence, but not one we're interested in.
                _ if last != [0x00, 0x00] => {
                    sout.extend(last.iter().skip_while(|b| **b == 0x00));
                    sout.push(*ch);
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }

                // Escape single quotes by doubling them.
                b'\'' => {
                    // Single quotation mark (U+0027).
                    sout.extend(b"''");
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }
                _ => {
                    sout.push(*ch); // Otherwise, just copy the byte.
                    last = [0x00, 0x00]; // Reset last byte tracker.
                }
            }
            sout
        });
        sout.push(b'\'');
    }
}

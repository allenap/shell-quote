//! <!-- Override references in README.md back to `cargo doc`-generated links. -->
//!
//! [`&str`]: `&str`
//! [`String`]: `String`
//! [`bstr::BStr`]: `bstr::BStr`
//! [`bstr::BString`]: `bstr::BString`
//! [`slice`]: `slice`
//! [`Vec<u8>`]: `std::vec::Vec<u8>`
//! [`OsStr`]: `std::ffi::OsStr`
//! [`OsString`]: `std::ffi::OsString`
//! [`Path`]: `std::path::Path`
//! [`PathBuf`]: `std::path::PathBuf`
//!
//! [`Sh`]: `Sh`
//! [`Dash`]: `Dash`
//! [`Bash`]: `Bash`
//! [`Fish`]: `Fish`
//! [`Zsh`]: `Zsh`
//!
//! [`QuoteRefExt`]: `QuoteRefExt`
//! [`QuoteRefExt::quoted`]: `QuoteRefExt::quoted`
//! [`QuoteExt`]: `QuoteExt`
//!
//! <style>
//!   .readme-only { display: none; }
//! </style>
//!

#![cfg_attr(
    all(
        feature = "bstr",
        feature = "bash",
        feature = "fish",
        feature = "sh",
    ),
    doc = include_str!("../README.md")
)]

mod ascii;
mod bash;
mod fish;
mod sh;
mod utf8;

#[cfg(feature = "bash")]
pub use bash::Bash;
#[cfg(feature = "fish")]
pub use fish::Fish;
#[cfg(feature = "sh")]
pub use sh::Sh;

/// Dash accepts the same quoted/escaped strings as `/bin/sh` – indeed, on many
/// systems, `dash` _is_ `/bin/sh` – hence this is an alias for [`Sh`].
#[cfg(feature = "sh")]
pub type Dash = sh::Sh;

/// Zsh accepts the same quoted/escaped strings as Bash, hence this is an alias
/// for [`Bash`].
#[cfg(feature = "bash")]
pub type Zsh = bash::Bash;

// ----------------------------------------------------------------------------

/// Quoting/escaping a string of bytes into a shell-safe form.
pub trait QuoteInto<OUT: ?Sized> {
    /// Quote/escape a string of bytes into an existing container.
    fn quote_into<'q, S: Into<Quotable<'q>>>(s: S, out: &mut OUT);
}

/// Quoting/escaping a string of bytes into a shell-safe form.
pub trait Quote<OUT: Default>: QuoteInto<OUT> {
    /// Quote/escape a string of bytes into a new container.
    fn quote<'q, S: Into<Quotable<'q>>>(s: S) -> OUT {
        let mut out = OUT::default();
        Self::quote_into(s, &mut out);
        out
    }
}

/// Blanket [`Quote`] impl for anything that has a [`QuoteInto`] impl.
impl<T: QuoteInto<OUT>, OUT: Default> Quote<OUT> for T {}

// ----------------------------------------------------------------------------

/// Extension trait for pushing shell quoted byte slices, e.g. `&[u8]`, [`&str`]
/// – anything that's [`Quotable`] – into container types like [`Vec<u8>`],
/// [`String`], [`OsString`] on Unix, and [`bstr::BString`] if it's enabled.
pub trait QuoteExt {
    fn push_quoted<'q, Q, S>(&mut self, _q: Q, s: S)
    where
        Q: QuoteInto<Self>,
        S: Into<Quotable<'q>>;
}

impl<T: ?Sized> QuoteExt for T {
    fn push_quoted<'q, Q, S>(&mut self, _q: Q, s: S)
    where
        Q: QuoteInto<Self>,
        S: Into<Quotable<'q>>,
    {
        Q::quote_into(s, self);
    }
}

// ----------------------------------------------------------------------------

/// Extension trait for shell quoting many different owned and reference types,
/// e.g. `&[u8]`, [`&str`] – anything that's [`Quotable`] – into owned container
/// types like [`Vec<u8>`], [`String`], [`OsString`] on Unix, and
/// [`bstr::BString`] if it's enabled.
pub trait QuoteRefExt<Output: Default> {
    fn quoted<Q: Quote<Output>>(self, q: Q) -> Output;
}

impl<'a, S, OUT: Default> QuoteRefExt<OUT> for S
where
    S: Into<Quotable<'a>>,
{
    fn quoted<Q: Quote<OUT>>(self, _q: Q) -> OUT {
        Q::quote(self)
    }
}

// ----------------------------------------------------------------------------

/// A string of bytes that can be quoted/escaped.
///
/// This is used by many methods in this crate as a generic
/// [`Into<Quotable>`][`Into`] constraint. Why not accept
/// [`AsRef<[u8]>`][`AsRef`] instead? The ergonomics of that approach were not
/// so good. For example, quoting [`OsString`]/[`OsStr`] and
/// [`PathBuf`]/[`Path`] didn't work in a natural way.
pub enum Quotable<'a> {
    #[cfg_attr(
        not(any(feature = "bash", feature = "fish", feature = "sh")),
        allow(unused)
    )]
    Bytes(&'a [u8]),
    #[cfg_attr(
        not(any(feature = "bash", feature = "fish", feature = "sh")),
        allow(unused)
    )]
    Text(&'a str),
}

impl<'a> From<&'a [u8]> for Quotable<'a> {
    fn from(source: &'a [u8]) -> Quotable<'a> {
        Quotable::Bytes(source)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Quotable<'a> {
    fn from(source: &'a [u8; N]) -> Quotable<'a> {
        Quotable::Bytes(&source[..])
    }
}

impl<'a> From<&'a Vec<u8>> for Quotable<'a> {
    fn from(source: &'a Vec<u8>) -> Quotable<'a> {
        Quotable::Bytes(source)
    }
}

impl<'a> From<&'a str> for Quotable<'a> {
    fn from(source: &'a str) -> Quotable<'a> {
        Quotable::Text(source)
    }
}

impl<'a> From<&'a String> for Quotable<'a> {
    fn from(source: &'a String) -> Quotable<'a> {
        Quotable::Text(source)
    }
}

#[cfg(feature = "bstr")]
impl<'a> From<&'a bstr::BStr> for Quotable<'a> {
    fn from(source: &'a bstr::BStr) -> Quotable<'a> {
        let bytes: &[u8] = source.as_ref();
        bytes.into()
    }
}

#[cfg(feature = "bstr")]
impl<'a> From<&'a bstr::BString> for Quotable<'a> {
    fn from(source: &'a bstr::BString) -> Quotable<'a> {
        let bytes: &[u8] = source.as_ref();
        bytes.into()
    }
}

// ----------------------------------------------------------------------------

/// Unix-specific support.
#[cfg(unix)]
mod platform {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    use super::Quotable;

    impl<'a> From<&'a OsStr> for Quotable<'a> {
        fn from(source: &'a OsStr) -> Quotable<'a> {
            use std::os::unix::ffi::OsStrExt;
            source.as_bytes().into()
        }
    }

    impl<'a> From<&'a OsString> for Quotable<'a> {
        fn from(source: &'a OsString) -> Quotable<'a> {
            use std::os::unix::ffi::OsStrExt;
            source.as_bytes().into()
        }
    }

    impl<'a> From<&'a Path> for Quotable<'a> {
        fn from(source: &'a Path) -> Quotable<'a> {
            source.as_os_str().into()
        }
    }

    impl<'a> From<&'a PathBuf> for Quotable<'a> {
        fn from(source: &'a PathBuf) -> Quotable<'a> {
            source.as_os_str().into()
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg(windows)]
pub use platform::UnicodeError;

/// Windows-specific support.
#[cfg(windows)]
mod platform {
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    use super::Quotable;

    /// Error when converting an [`OsStr`]/[`OsString`] that contains invalid
    /// Unicode, e.g. unpaired UTF-16 surrogates on Windows.
    #[derive(Debug, Clone)]
    #[non_exhaustive]
    pub struct UnicodeError {}

    impl std::error::Error for UnicodeError {}

    impl std::fmt::Display for UnicodeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "invalid Unicode (unpaired UTF-16 surrogates)")
        }
    }

    impl<'a> TryFrom<&'a OsStr> for Quotable<'a> {
        type Error = UnicodeError;

        fn try_from(source: &'a OsStr) -> Result<Self, Self::Error> {
            source.to_str().map(Quotable::Text).ok_or(UnicodeError {})
        }
    }

    impl<'a> TryFrom<&'a OsString> for Quotable<'a> {
        type Error = UnicodeError;

        fn try_from(source: &'a OsString) -> Result<Self, Self::Error> {
            source.as_os_str().try_into()
        }
    }

    impl<'a> TryFrom<&'a Path> for Quotable<'a> {
        type Error = UnicodeError;

        fn try_from(source: &'a Path) -> Result<Self, Self::Error> {
            source.as_os_str().try_into()
        }
    }

    impl<'a> TryFrom<&'a PathBuf> for Quotable<'a> {
        type Error = UnicodeError;

        fn try_from(source: &'a PathBuf) -> Result<Self, Self::Error> {
            source.as_os_str().try_into()
        }
    }
}

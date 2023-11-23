#![doc = include_str!("../README.md")]

use std::{
    ffi::{OsStr, OsString},
    str::Utf8Error,
    string::FromUtf8Error,
};

mod ascii;
pub mod bash;
pub mod sh;

pub use bash::Bash;
pub use sh::Sh;

pub trait QuoteExt {
    fn push_quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(&mut self, shell: Q, s: &S);
    fn quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(shell: Q, s: &S) -> Self;
}

impl QuoteExt for Vec<u8> {
    fn push_quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(&mut self, _shell: Q, s: &S) {
        Q::quote_into(s, self);
    }

    fn quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(_shell: Q, s: &S) -> Self {
        Q::quote(s)
    }
}

#[cfg(unix)]
impl QuoteExt for OsString {
    fn push_quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(&mut self, _shell: Q, s: &S) {
        use std::os::unix::ffi::OsStrExt;
        let quoted = Q::quote(s);
        self.push(OsStr::from_bytes(&quoted))
    }

    fn quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(_shell: Q, s: &S) -> Self {
        use std::os::unix::ffi::OsStringExt;
        let quoted = Q::quote(s);
        OsString::from_vec(quoted)
    }
}

pub trait StringQuoteExt {
    fn push_quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(
        &mut self,
        shell: Q,
        s: &S,
    ) -> Result<(), Utf8Error>;

    fn quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(shell: Q, s: &S) -> Result<Self, FromUtf8Error>
    where
        Self: Sized;
}

impl StringQuoteExt for String {
    fn push_quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(
        &mut self,
        _shell: Q,
        s: &S,
    ) -> Result<(), Utf8Error> {
        let quoted = Q::quote(s);
        self.push_str(std::str::from_utf8(&quoted)?);
        Ok(())
    }

    fn quoted<Q: Quoter, S: ?Sized + AsRef<[u8]>>(_shell: Q, s: &S) -> Result<Self, FromUtf8Error> {
        let quoted = Q::quote(s);
        String::from_utf8(quoted)
    }
}

pub trait Quoter {
    /// Quote/escape a string of bytes into a new `Vec<u8>`.
    fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8>;

    /// Quote/escape a string of bytes into an existing `Vec<u8>`.
    fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>);
}

// ----------------------------------------------------------------------------

#[cfg(test)]
pub(crate) fn find_bins<P: AsRef<std::path::Path>>(name: P) -> Vec<std::path::PathBuf> {
    let name = name.as_ref();
    match std::env::var_os("PATH") {
        Some(path) => {
            // Find every `name` file in `path`, return as absolute paths.
            std::env::split_paths(&path)
                .map(|bindir| bindir.join(name))
                .filter(|bin| bin.exists())
                .collect()
        }
        None => {
            // Return the bare name. If the calling test executes this it will
            // likely fail. This is desirable: we want the test to fail.
            vec![name.into()]
        }
    }
}

#![doc = include_str!("../README.md")]

use std::ffi::{OsStr, OsString};

pub mod bash;
pub mod sh;

mod ascii;

pub trait ShellQuoter {
    /// Quote/escape a string of bytes into a new `Vec<u8>`.
    fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8>;

    /// Quote/escape a string of bytes into an existing `Vec<u8>`.
    fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>);
}

pub struct Bash;

impl ShellQuoter for Bash {
    fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8> {
        bash::quote(s)
    }

    fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>) {
        bash::quote_into(s, sout)
    }
}

pub struct Sh;

impl ShellQuoter for Sh {
    fn quote<S: ?Sized + AsRef<[u8]>>(s: &S) -> Vec<u8> {
        sh::quote(s)
    }

    fn quote_into<S: ?Sized + AsRef<[u8]>>(s: &S, sout: &mut Vec<u8>) {
        sh::quote_into(s, sout)
    }
}

pub trait ShellQuoteExt {
    fn push_quoted<Q: ShellQuoter, S: ?Sized + AsRef<[u8]>>(&mut self, shell: Q, s: &S);
}

impl ShellQuoteExt for Vec<u8> {
    fn push_quoted<Q: ShellQuoter, S: ?Sized + AsRef<[u8]>>(&mut self, _shell: Q, s: &S) {
        Q::quote_into(s, self);
    }
}

#[cfg(unix)]
impl ShellQuoteExt for OsString {
    fn push_quoted<Q: ShellQuoter, S: ?Sized + AsRef<[u8]>>(&mut self, _shell: Q, s: &S) {
        use std::os::unix::ffi::OsStrExt;
        let quoted = Q::quote(s);
        self.push(OsStr::from_bytes(&quoted))
    }
}

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

#[cfg(test)]
mod tests {
    use super::{Bash, Sh, ShellQuoteExt};
    use std::ffi::OsString;

    #[test]
    fn test_push_quoted_with_bash() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap();
        assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_push_quoted_with_sh() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap();
        assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted_with_bash() {
        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap();
        assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted_with_sh() {
        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap();
        assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
    }
}

#![cfg(feature = "sh")]

mod util;

// -- Helpers -----------------------------------------------------------------

use std::{
    ffi::OsStr,
    io,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Output},
};

pub(crate) fn invoke_bash_as_sh(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin)
        .arg0("sh")
        .arg("--posix")
        .arg("-c")
        .arg(script)
        .output()
}

pub(crate) fn invoke_zsh_as_sh(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin)
        .arg0("sh")
        .arg("--emulate")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .output()
}

// -- impl Sh -----------------------------------------------------------------

mod sh_impl {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    use super::util::{find_bins, invoke_shell};
    use super::{invoke_bash_as_sh, invoke_zsh_as_sh};
    use shell_quote::Sh;

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
        assert_eq!(Sh::quote("-_=/,.+"), b"-_'=/,.+'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Sh::quote(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Sh::quote(r#"woo'wah""#), br#"woo\'wah'"'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Sh::quote("\x07"), b"'\x07'");
        assert_eq!(Sh::quote("\x00"), b"'\x00'");
        assert_eq!(Sh::quote("\x06"), b"'\x06'");
        assert_eq!(Sh::quote("\x7F"), b"'\x7F'");
        assert_eq!(Sh::quote("\x1B"), b"'\x1B'");
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
        assert_eq!(buffer, b"-_'=/,.+'");
    }

    fn script() -> (OsString, OsString) {
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let input: OsString = OsString::from_vec((1..=u8::MAX).collect());
        // NOTE: Do NOT use `echo` here; in most/all shells it interprets
        // escapes with no way to disable that behaviour (unlike the `echo`
        // builtin in Bash, for example, which accepts a `-E` flag). Using
        // `printf %s` seems to do the right thing in most shells, i.e. it does
        // not interpret the arguments in any way.
        let mut script = b"printf %s ".to_vec();
        Sh::quote_into(input.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        (input, script)
    }

    #[test]
    fn test_roundtrip_sh() {
        let (input, script) = script();
        for bin in find_bins("sh") {
            let output = invoke_shell(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[test]
    fn test_roundtrip_dash() {
        let (input, script) = script();
        for bin in find_bins("dash") {
            let output = invoke_shell(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[test]
    fn test_roundtrip_bash() {
        let (input, script) = script();
        for bin in find_bins("bash") {
            let output = invoke_shell(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[test]
    fn test_roundtrip_bash_as_sh() {
        let (input, script) = script();
        for bin in find_bins("bash") {
            let output = invoke_bash_as_sh(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[test]
    fn test_roundtrip_zsh() {
        let (input, script) = script();
        for bin in find_bins("zsh") {
            let output = invoke_shell(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[test]
    fn test_roundtrip_zsh_as_sh() {
        let (input, script) = script();
        for bin in find_bins("zsh") {
            let output = invoke_zsh_as_sh(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod sh_quote_ext {
    use shell_quote::{QuoteExt, Sh};

    #[test]
    fn test_vec_push_quoted() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted() {
        use std::ffi::OsString;

        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted() {
        let mut string: bstr::BString = "Hello, ".into();
        string.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted_roundtrip() {
        use super::util::{find_bins, invoke_shell};
        use bstr::{BString, ByteSlice};
        let mut script: BString = "printf %s ".into();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: Vec<_> = (1..=u8::MAX).collect();
        script.push_quoted(Sh, &string);
        let script = script.to_os_str().unwrap();
        // Test with every version of `sh` we find on `PATH`.
        for bin in find_bins("sh") {
            let output = invoke_shell(&bin, script).unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

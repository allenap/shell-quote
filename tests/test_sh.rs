// -- Helpers -----------------------------------------------------------------

use std::{
    ffi::OsStr,
    io,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Output},
};

mod util;

fn invoke_sh(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin).arg("-c").arg(script).output()
}

fn invoke_dash(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin).arg("-c").arg(script).output()
}

fn invoke_bash_as_sh(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin)
        .arg0("sh")
        .arg("--posix")
        .arg("-c")
        .arg(script)
        .output()
}

fn invoke_zsh_as_sh(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin)
        .arg0("sh")
        .arg("--emulate")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .output()
}

// -- impl Sh -----------------------------------------------------------------

mod impl_sh {
    use super::util;
    use super::{invoke_bash_as_sh, invoke_dash, invoke_sh, invoke_zsh_as_sh};
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

    #[test]
    fn test_roundtrip() {
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: OsString = OsString::from_vec((1..=u8::MAX).collect());
        // NOTE: Do NOT use `echo` here; it interprets escapes with no way to
        // disable that behaviour (unlike the `echo` builtin in Bash, for
        // example, which accepts a `-E` flag).
        let mut script = b"printf '%s' ".to_vec();
        Sh::quote_into(string.as_bytes(), &mut script);
        let script = OsString::from_vec(script);

        for bin in util::find_bins("sh") {
            let output = invoke_sh(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, string);
        }

        for bin in util::find_bins("dash") {
            let output = invoke_dash(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, string);
        }

        for bin in util::find_bins("bash") {
            let output = invoke_bash_as_sh(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, string);
        }

        for bin in util::find_bins("zsh") {
            let output = invoke_zsh_as_sh(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, string);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod quote_ext {
    use super::util;
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

    #[test]
    fn test_string_push_quoted() {
        let mut string: String = "Hello, ".into();
        string.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_string_push_quoted_roundtrip() {
        use std::process::Command;

        let mut script = "printf '%s' ".to_owned();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: Vec<_> = (1..=u8::MAX).collect();
        script.push_quoted(Sh, &string);
        // Test with every version of `sh` we find on `PATH`.
        for bin in util::find_bins("sh") {
            let output = Command::new(bin).arg("-c").arg(&script).output().unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

#![cfg(feature = "bash")]

mod util;

// -- impl Bash ---------------------------------------------------------------

mod bash_impl {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    use super::util::{find_bins, invoke_shell};
    use shell_quote::Bash;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            Bash::quote("abcdefghijklmnopqrstuvwxyz"),
            b"abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            Bash::quote("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(Bash::quote("0123456789"), b"0123456789");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(Bash::quote("-_=/,.+"), b"$'-_=/,.+'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Bash::quote(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Bash::quote(r#"woo"wah""#), br#"$'woo"wah"'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Bash::quote("\x00"), b"$'\\x00'");
        assert_eq!(Bash::quote("\x07"), b"$'\\a'");
        assert_eq!(Bash::quote("\x00"), b"$'\\x00'");
        assert_eq!(Bash::quote("\x06"), b"$'\\x06'");
        assert_eq!(Bash::quote("\x7F"), b"$'\\x7F'");
    }

    #[test]
    fn test_escape_into_plain() {
        let mut buffer = Vec::new();
        Bash::quote_into("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_escape_into_with_escapes() {
        let mut buffer = Vec::new();
        Bash::quote_into("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"$'-_=/,.+'");
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
        Bash::quote_into(input.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        (input, script)
    }

    #[test]
    fn test_roundtrip_bash() {
        let (input, script) = script();
        for bin in find_bins("bash") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, input);
        }
    }

    #[test]
    fn test_roundtrip_zsh() {
        let (input, script) = script();
        for bin in find_bins("zsh") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, input);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod bash_quote_ext {
    use super::util::{find_bins, invoke_shell};
    use shell_quote::{Bash, QuoteExt};

    #[test]
    fn test_vec_push_quoted_with_bash() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted_with_bash() {
        use std::ffi::OsString;

        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_string_push_quoted_with_bash() {
        let mut string: String = "Hello, ".into();
        string.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_string_push_quoted_roundtrip() {
        let mut script = "printf %s ".to_owned();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: Vec<_> = (1u8..=u8::MAX).collect();
        script.push_quoted(Bash, &string);
        // Test with every version of `bash` we find on `PATH`.
        for bin in find_bins("bash") {
            let output = invoke_shell(&bin, script.as_ref()).unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

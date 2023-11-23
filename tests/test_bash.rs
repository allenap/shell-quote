mod util;

// -- impl Bash ---------------------------------------------------------------

mod impl_bash {
    use super::util;
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

    #[test]
    fn test_roundtrip() {
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        use std::process::Command;

        let mut script = b"echo -n ".to_vec();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C. To me, seems like a bug in Bash.
        let string: OsString = OsString::from_vec((1u8..=u8::MAX).collect());
        Bash::quote_into(string.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        // Test with every version of `bash` we find on `PATH`.
        for bin in util::find_bins("bash") {
            let output = Command::new(bin).arg("-c").arg(&script).output().unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, string);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod quote_ext {
    use super::util;
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
        use std::process::Command;

        let mut script = "echo -n ".to_owned();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C. To me, seems like a bug in Bash.
        let string: Vec<_> = (1u8..=u8::MAX).collect();
        script.push_quoted(Bash, &string);
        // Test with every version of `bash` we find on `PATH`.
        for bin in util::find_bins("bash") {
            let output = Command::new(bin).arg("-c").arg(&script).output().unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

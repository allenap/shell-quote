mod util;

// -- impl Fish ---------------------------------------------------------------

mod impl_fish {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    use super::util::{find_bins, invoke_shell};
    use shell_quote::Fish;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            Fish::quote("abcdefghijklmnopqrstuvwxyz"),
            b"abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            Fish::quote("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(Fish::quote("0123456789"), b"0123456789");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(Fish::quote("-_=/,.+"), b"-_'=/,.+'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Fish::quote(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Fish::quote(r#"woo"wah""#), br#"woo'"wah"'"#);
        assert_eq!(Fish::quote(r#"'"#), br#"\'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Fish::quote("\x00"), b"\\x00");
        assert_eq!(Fish::quote("\x07"), b"\\a");
        assert_eq!(Fish::quote("\x00"), b"\\x00");
        assert_eq!(Fish::quote("\x06"), b"\\x06");
        assert_eq!(Fish::quote("\x7F"), b"\\x7F");
    }

    #[test]
    fn test_multiple_parts() {
        assert_eq!(Fish::quote("\x00AABB"), b"\\x00AABB");
        assert_eq!(Fish::quote("\x07A\x06B\x07"), b"\\aA\\x06B\\a");
        assert_eq!(Fish::quote("AAA\x7F"), b"AAA\\x7F");
        assert_eq!(Fish::quote("\x06\x06"), b"\\x06\\x06");
    }

    #[test]
    fn test_new_lines() {
        assert_eq!(Fish::quote("A\nB"), b"A\\nB");
    }

    #[test]
    fn test_escape_into_plain() {
        let mut buffer = Vec::new();
        Fish::quote_into("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_escape_into_with_escapes() {
        let mut buffer = Vec::new();
        Fish::quote_into("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"-_'=/,.+'");
    }

    #[test]
    fn test_roundtrip() {
        let mut script = b"echo -n ".to_vec();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: OsString = OsString::from_vec((1u8..=u8::MAX).collect());
        Fish::quote_into(string.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, string);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod fish_quote_ext {
    use super::util::{find_bins, invoke_shell};
    use shell_quote::{Fish, QuoteExt};

    #[test]
    fn test_vec_push_quoted_with_fish() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Fish, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted_with_fish() {
        use std::ffi::OsString;

        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Fish, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_string_push_quoted_with_fish() {
        let mut string: String = "Hello, ".into();
        string.push_quoted(Fish, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[test]
    fn test_string_push_quoted_roundtrip() {
        let mut script = "echo -n ".to_owned();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: Vec<_> = (1u8..=u8::MAX).collect();
        script.push_quoted(Fish, &string);
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, script.as_ref()).unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

#![cfg(feature = "fish")]

mod resources;
mod util;

// -- impl Fish ---------------------------------------------------------------

mod fish_impl {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    use crate::resources;

    use super::util::{find_bins, invoke_shell};
    use shell_quote::Fish;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            Fish::quote_vec("abcdefghijklmnopqrstuvwxyz"),
            b"abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            Fish::quote_vec("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(Fish::quote_vec("0123456789"), b"0123456789");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(Fish::quote_vec("-_=/,.+"), b"-_'=/,.+'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Fish::quote_vec(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Fish::quote_vec(r#"woo"wah""#), br#"woo'"wah"'"#);
        assert_eq!(Fish::quote_vec(r#"'"#), br#"\'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Fish::quote_vec("\x00"), b"\\X00");
        assert_eq!(Fish::quote_vec("\x07"), b"\\a");
        assert_eq!(Fish::quote_vec("\x00"), b"\\X00");
        assert_eq!(Fish::quote_vec("\x06"), b"\\X06");
        assert_eq!(Fish::quote_vec("\x7F"), b"\\X7F");
    }

    #[test]
    fn test_utf8() {
        // UTF-8 for code points U+0080 and above is included verbatim.
        assert_eq!(Fish::quote_vec("Hello 👋"), b"Hello' \xf0\x9f\x91\x8b'");
    }

    #[test]
    fn test_multiple_parts() {
        assert_eq!(Fish::quote_vec("\x00AA12"), b"\\X00AA12");
        assert_eq!(Fish::quote_vec("\x07A\x06B\x07"), b"\\aA\\X06B\\a");
        assert_eq!(Fish::quote_vec("AAA\x7F"), b"AAA\\X7F");
        assert_eq!(Fish::quote_vec("\x06\x06"), b"\\X06\\X06");
    }

    #[test]
    fn test_new_lines() {
        assert_eq!(Fish::quote_vec("A\nB"), b"A\\nB");
    }

    #[test]
    fn test_escape_into_plain() {
        let mut buffer = Vec::new();
        Fish::quote_into_vec("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_escape_into_with_escapes() {
        let mut buffer = Vec::new();
        Fish::quote_into_vec("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"-_'=/,.+'");
    }

    #[test]
    fn test_roundtrip_bytes() {
        // Unlike many/most other shells, `echo` is safe here because backslash
        // escapes are _not_ interpreted by default.
        let mut script = b"echo -n -- ".to_vec();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let input: OsString = OsString::from_vec((1u8..=u8::MAX).collect());
        Fish::quote_into_vec(input.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, input);
        }
    }

    #[test]
    fn test_roundtrip_text() {
        // Unlike many/most other shells, `echo` is safe here because backslash
        // escapes are _not_ interpreted by default.
        let mut script = b"echo -n -- ".to_vec();
        Fish::quote_into_vec(resources::UTF8_SAMPLE, &mut script);
        let input: OsString = resources::UTF8_SAMPLE.into();
        let script = OsString::from_vec(script);
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, input);
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod fish_quote_ext {
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

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted_with_fish() {
        let mut string: bstr::BString = "Hello, ".into();
        string.push_quoted(Fish, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, World,' Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted_roundtrip() {
        use super::util::{find_bins, invoke_shell};
        use bstr::{BString, ByteSlice};
        // Unlike many/most other shells, `echo` is safe here because backslash
        // escapes are _not_ interpreted by default.
        let mut script: BString = "echo -n -- ".into();
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let string: Vec<_> = (1u8..=u8::MAX).collect();
        script.push_quoted(Fish, &string);
        let script = script.to_os_str().unwrap();
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, script).unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

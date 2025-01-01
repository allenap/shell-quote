#![cfg(feature = "fish")]

mod resources;
mod util;

// -- Helpers -----------------------------------------------------------------

use std::{path::Path, process::Command};

fn fish_version(bin: &Path) -> semver::Version {
    let script = "printf %s $version";
    let output = Command::new(bin).arg("-c").arg(script).output().unwrap();
    let version = String::from_utf8(output.stdout).unwrap();
    lenient_semver::parse(&version).unwrap()
}

/// The `\\XHH` format (backslash, a literal "X", two hex characters) is
/// understood by fish. The `\\xHH` format is _also_ understood, but until fish
/// 3.6.0 it had a weirdness. From the [release notes][]:
///
/// > The `\\x` and `\\X` escape syntax is now equivalent. `\\xAB` previously
/// > behaved the same as `\\XAB`, except that it would error if the value “AB”
/// > was larger than “7f” (127 in decimal, the highest ASCII value).
///
/// [release notes]: https://github.com/fish-shell/fish-shell/releases/tag/3.6.0
static _FISH_VERSION_ESCAPE_SYNTAX_FIXED: semver::Version = semver::Version::new(3, 6, 0);

/// fish couldn't correctly deal with some Unicode code points encoded as UTF-8
/// prior to version 3.6.2. From the [release notes][]:
///
/// > fish uses certain Unicode non-characters internally for marking wildcards
/// > and expansions. It incorrectly allowed these markers to be read on command
/// > substitution output, rather than transforming them into a safe internal
/// > representation.
///
/// [release notes]: https://github.com/fish-shell/fish-shell/releases/tag/3.6.2
static FISH_VERSION_UNICODE_FIXED: semver::Version = semver::Version::new(3, 6, 2);

// -- impl Fish ---------------------------------------------------------------

mod fish_impl {
    use std::ffi::OsString;

    use super::{
        resources,
        util::{find_bins, invoke_shell},
    };
    use shell_quote::Fish;
    use test_case::test_matrix;

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
        assert_eq!(Fish::quote_vec("Hello \r\n"), b"Hello' '\\r\\n");
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

    #[cfg(unix)]
    #[test_matrix((script_bytes, script_text))]
    fn test_roundtrip(prepare: fn() -> (OsString, OsString)) {
        use std::os::unix::ffi::OsStringExt;
        let (input, script) = prepare();
        // Test with every version of `fish` we find on `PATH`.
        for bin in find_bins("fish") {
            let output = invoke_shell(&bin, &script).unwrap();
            let result = OsString::from_vec(output.stdout);
            assert_eq!(result, input);
        }
    }

    #[cfg(unix)]
    fn script_bytes() -> (OsString, OsString) {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        // It doesn't seem possible to roundtrip NUL, probably because it is the
        // string terminator character in C.
        let input: OsString = OsString::from_vec((1..=u8::MAX).collect());
        // Unlike many/most other shells, `echo` is safe here because backslash
        // escapes are _not_ interpreted by default.
        let mut script = b"echo -n -- ".to_vec();
        Fish::quote_into_vec(input.as_bytes(), &mut script);
        let script = OsString::from_vec(script);
        (input, script)
    }

    #[cfg(unix)]
    fn script_text() -> (OsString, OsString) {
        use std::os::unix::ffi::OsStringExt;
        // Unlike many/most other shells, `echo` is safe here because backslash
        // escapes are _not_ interpreted by default.
        let mut script = b"echo -n -- ".to_vec();
        Fish::quote_into_vec(resources::UTF8_SAMPLE, &mut script);
        let script = OsString::from_vec(script);
        (resources::UTF8_SAMPLE.into(), script)
    }

    #[cfg(unix)]
    #[test]
    fn test_roundtrip_utf8_full() {
        use std::os::unix::ffi::OsStringExt;
        let utf8: Vec<_> = ('\x01'..=char::MAX).collect(); // Not including NUL.
        for bin in find_bins("fish") {
            let version = super::fish_version(&bin);
            if version < super::FISH_VERSION_UNICODE_FIXED {
                eprintln!("Skipping fish {version}; it's broken. See FISH_VERSION_UNICODE_FIXED.");
                continue;
            }
            // Chunk to avoid over-length arguments (see`getconf ARG_MAX`).
            for chunk in utf8.chunks(2usize.pow(14)) {
                let input: String = String::from_iter(chunk);
                let mut script = b"printf %s ".to_vec();
                Fish::quote_into_vec(&input, &mut script);
                let script = OsString::from_vec(script);
                let output = invoke_shell(&bin, &script).unwrap();
                let observed = OsString::from_vec(output.stdout);
                assert_eq!(observed.into_string(), Ok(input));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    /// IIRC, this caught bugs not found by `test_roundtrip_utf8_full`, and it
    /// was much easier to figure out what the failures meant. For now it stays!
    fn test_roundtrip_utf8_full_char_by_char() {
        use std::os::unix::ffi::OsStringExt;
        let utf8: Vec<_> = ('\x01'..=char::MAX).collect(); // Not including NUL.
        for bin in find_bins("fish") {
            let version = super::fish_version(&bin);
            if version < super::FISH_VERSION_UNICODE_FIXED {
                eprintln!("Skipping fish {version}; it's broken. See FISH_VERSION_UNICODE_FIXED.");
                continue;
            }
            // Chunk to avoid over-length arguments (see`getconf ARG_MAX`).
            for chunk in utf8.chunks(2usize.pow(12)) {
                let script = OsString::from_vec(chunk.iter().fold(
                    b"printf '%s\\0'".to_vec(),
                    |mut script, ch| {
                        script.push(b' ');
                        Fish::quote_into_vec(&ch.to_string(), &mut script);
                        script
                    },
                ));

                let output = invoke_shell(&bin, &script).unwrap();
                let observed = output.stdout.split(|ch| *ch == 0).filter(|s| !s.is_empty());
                assert_eq!(observed.clone().count(), chunk.len());

                for (ob, ex) in observed.zip(chunk) {
                    let ob_str = std::str::from_utf8(ob).unwrap();
                    assert_eq!(ob_str.chars().count(), 1, "char count mismatch");
                    let ob_char = ob_str.chars().next().unwrap();
                    assert_eq!(ob_char, *ex, "chars do not match");
                }
            }
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

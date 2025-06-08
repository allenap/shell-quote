#![cfg(all(unix, feature = "pwsh"))]

mod resources;
mod util;

// -- impl Pwsh ---------------------------------------------------------------

mod pwsh_impl {
    use std::ffi::{OsStr, OsString};
    use std::{io::Result, path::Path, process::Output};

    use super::resources;
    use super::util::{find_bins, invoke_shell};
    use shell_quote::Pwsh;
    use test_case::test_matrix;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            Pwsh::quote_vec("abcdefghijklmnopqrstuvwxyz"),
            b"'abcdefghijklmnopqrstuvwxyz'"
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            Pwsh::quote_vec("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            b"'ABCDEFGHIJKLMNOPQRSTUVWXYZ'"
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(Pwsh::quote_vec("0123456789"), b"'0123456789'");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(Pwsh::quote_vec("-_=/,.+"), b"'-_=/,.+'");
        assert_eq!(Pwsh::quote_vec("Hello \r\n"), b"'Hello \r\n'");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Pwsh::quote_vec(""), b"''");
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(Pwsh::quote_vec(r#"woo'wah""#), br#"'woo''wah"'"#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(Pwsh::quote_vec("\x07"), b"'\x07'");
        assert_eq!(Pwsh::quote_vec("\x00"), b"'\x00'");
        assert_eq!(Pwsh::quote_vec("\x06"), b"'\x06'");
        assert_eq!(Pwsh::quote_vec("\x7F"), b"'\x7F'");
        assert_eq!(Pwsh::quote_vec("\x1B"), b"'\x1B'");
    }

    #[test]
    fn test_quote_into_plain() {
        let mut buffer = Vec::new();
        Pwsh::quote_into_vec("hello", &mut buffer);
        assert_eq!(buffer, b"'hello'");
    }

    #[test]
    fn test_quote_into_with_escapes() {
        let mut buffer = Vec::new();
        Pwsh::quote_into_vec("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"'-_=/,.+'");
    }

    type InvokeShell = fn(&Path, &OsStr) -> Result<Output>;

    #[cfg(unix)]
    #[test_matrix(
        (script_bytes,
         script_text),
        (("pwsh", invoke_shell),)
    )]
    fn test_roundtrip(prepare: fn() -> (OsString, OsString), (shell, invoke): (&str, InvokeShell)) {
        use std::os::unix::ffi::OsStringExt;
        let (input, script) = prepare();
        for bin in find_bins(shell) {
            let output = invoke(&bin, &script).unwrap();
            let observed = OsString::from_vec(output.stdout);
            assert_eq!(observed, input);
        }
    }

    #[cfg(unix)]
    fn script_bytes() -> (OsString, OsString) {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        // Strings in PowerShell must be Unicode, so NUL and all high bytes
        // (>7f) are forbidden and either cause an error or result in the Unicode
        // Replacement Character � (U+FFFD).
        let input: OsString = OsString::from_vec((1..=0x7F).collect());
        // NOTE: Do NOT use `echo` here; in most/all shells it interprets
        // escapes with no way to disable that behaviour (unlike the `echo`
        // builtin in Bash, for example, which accepts a `-E` flag). Using
        // `printf %s` seems to do the right thing in most shells, i.e. it does
        // not interpret the arguments in any way.
        let mut script = b"[Console]::Write(".to_vec();
        Pwsh::quote_into_vec(input.as_bytes(), &mut script);
        script.push(b')');
        let script = OsString::from_vec(script);
        (input, script)
    }

    #[cfg(unix)]
    fn script_text() -> (OsString, OsString) {
        use std::os::unix::ffi::OsStringExt;
        let mut script = b"[Console]::Write(".to_vec();
        Pwsh::quote_into_vec(resources::UTF8_SAMPLE, &mut script);
        script.push(b')');
        let input: OsString = resources::UTF8_SAMPLE.into();
        let script = OsString::from_vec(script);
        (input, script)
    }

    #[cfg(unix)]
    #[test_matrix(
        (("pwsh", invoke_shell),)
    )]
    fn test_roundtrip_utf8_full((shell, invoke): (&str, InvokeShell)) {
        use std::os::unix::ffi::OsStringExt;
        let utf8: Vec<_> = ('\x01'..=char::MAX).collect(); // Not including NUL.
        for bin in find_bins(shell) {
            // Chunk to avoid over-length arguments (see`getconf ARG_MAX`).
            for chunk in utf8.chunks(2usize.pow(14)) {
                let input: String = String::from_iter(chunk);
                let mut script = b"[Console]::Write(".to_vec();
                Pwsh::quote_into_vec(&input, &mut script);
                script.push(b')');
                let script = OsString::from_vec(script);
                let output = invoke(&bin, &script).unwrap();
                let observed = OsString::from_vec(output.stdout);
                assert_eq!(observed.into_string(), Ok(input));
            }
        }
    }
}

// -- QuoteExt ----------------------------------------------------------------

mod pwsh_quote_ext {
    use std::ffi::OsString;

    use shell_quote::{Pwsh, QuoteExt};

    #[test]
    fn test_vec_push_quoted() {
        let mut buffer = Vec::from(b"Hello, ");
        buffer.push_quoted(Pwsh, "World, Bob, !@#$%^&*(){}[]");
        let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(unix)]
    #[test]
    fn test_os_string_push_quoted() {
        let mut buffer: OsString = "Hello, ".into();
        buffer.push_quoted(Pwsh, "World, Bob, !@#$%^&*(){}[]");
        let string = buffer.into_string().unwrap(); // -> test failures are more readable.
        assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted() {
        let mut string: bstr::BString = "Hello, ".into();
        string.push_quoted(Pwsh, "World, Bob, !@#$%^&*(){}[]");
        assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
    }

    #[cfg(feature = "bstr")]
    #[test]
    fn test_bstring_push_quoted_roundtrip() {
        use super::util::{find_bins, invoke_shell};
        use bstr::{BString, ByteSlice};
        let mut script: BString = "printf %s ".into();
        // Strings in PowerShell must be Unicode, so NUL and all high bytes
        // (>7f) are forbidden and either cause an error or result in the Unicode
        // Replacement Character � (U+FFFD).
        let string: Vec<_> = (1..=0x7F).collect();
        script.push_quoted(Pwsh, &string);
        let script = script.to_os_str().unwrap();
        // Test with every version of `pwsh` we find on `PATH`.
        for bin in find_bins("pwsh") {
            let output = invoke_shell(&bin, script).unwrap();
            assert_eq!(output.stdout, string);
        }
    }
}

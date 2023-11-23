#[test]
fn test_vec_push_quoted_with_bash() {
    use shell_quote::{QuoteExt, Sh};

    let mut buffer = Vec::from(b"Hello, ");
    buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
    let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
    assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
}

#[cfg(unix)]
#[test]
fn test_os_string_push_quoted_with_bash() {
    use shell_quote::{QuoteExt, Sh};
    use std::ffi::OsString;

    let mut buffer: OsString = "Hello, ".into();
    buffer.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
    let string = buffer.into_string().unwrap(); // -> test failures are more readable.
    assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
}

#[test]
fn test_string_push_quoted_with_bash() {
    use shell_quote::{QuoteExt, Sh};

    let mut string: String = "Hello, ".into();
    string.push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]");
    assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
}

#[test]
fn test_string_push_quoted_roundtrip() {
    use shell_quote::{QuoteExt, Sh};
    use std::process::Command;

    let mut script = "echo ".to_owned();
    // In Bash it doesn't seem possible to roundtrip NUL, but in the Bourne
    // shell, or whatever is masquerading as `sh`, it seems to be fine.
    let string: Vec<_> = (u8::MIN..=u8::MAX).collect();
    script.push_quoted(Sh, &string);
    let output = Command::new("sh").arg("-c").arg(&script).output().unwrap();
    let mut result = output.stdout;
    result.resize(result.len() - 1, 0); // Remove trailing newline.
    assert_eq!(result, string);
}

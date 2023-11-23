#[test]
fn test_vec_push_quoted_with_bash() {
    use shell_quote::{Bash, QuoteExt};

    let mut buffer = Vec::from(b"Hello, ");
    buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
    let string = String::from_utf8(buffer).unwrap(); // -> test failures are more readable.
    assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
}

#[cfg(unix)]
#[test]
fn test_os_string_push_quoted_with_bash() {
    use shell_quote::{Bash, QuoteExt};
    use std::ffi::OsString;

    let mut buffer: OsString = "Hello, ".into();
    buffer.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
    let string = buffer.into_string().unwrap(); // -> test failures are more readable.
    assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
}

#[test]
fn test_string_push_quoted_with_bash() {
    use shell_quote::{Bash, QuoteExt};

    let mut string: String = "Hello, ".into();
    string.push_quoted(Bash, "World, Bob, !@#$%^&*(){}[]");
    assert_eq!(string, "Hello, $'World, Bob, !@#$%^&*(){}[]'");
}

#[test]
fn test_string_push_quoted_roundtrip() {
    use shell_quote::{Bash, QuoteExt};
    use std::process::Command;

    let mut script = "echo -n ".to_owned();
    // It doesn't seem possible to roundtrip NUL, probably because it is the
    // string terminator character in C. To me this seems like a bug in Bash.
    let string: Vec<_> = (1u8..=u8::MAX).collect();
    script.push_quoted(Bash, &string);
    let output = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .output()
        .unwrap();
    assert_eq!(output.stdout, string);
}

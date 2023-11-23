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
    use shell_quote::{Sh, StringQuoteExt};

    let mut string: String = "Hello, ".into();
    string
        .push_quoted(Sh, "World, Bob, !@#$%^&*(){}[]")
        .unwrap();
    assert_eq!(string, "Hello, 'World, Bob, !@#$%^&*(){}[]'");
}

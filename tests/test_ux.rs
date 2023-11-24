use bstr::{BString, ByteSlice};
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

use shell_quote::{Bash, Quotable, QuoteRefExt};

#[test]
fn test_quotable_conversions() {
    let bytes = b"bytes";
    let byte_slice = &b"bytes"[..];
    let vec = Vec::from(b"vec");
    let os_string = OsString::from("os-string");
    let b_string = bstr::BString::from(b"b-string");
    let path_buf = std::path::PathBuf::from("/path/[to]/file");
    let string = "string".to_owned();

    let _: Quotable = bytes.into();
    let _: Quotable = byte_slice.into();
    let _: Quotable = (&vec).into();
    let _: Quotable = (&os_string).into();
    let _: Quotable = os_string.as_os_str().into();
    let _: Quotable = (&b_string).into();
    let _: Quotable = b_string.as_bstr().into();
    let _: Quotable = (&path_buf).into();
    let _: Quotable = path_buf.as_path().into();
    let _: Quotable = (&string).into();
    let _: Quotable = string.as_str().into();

    fn into_quotable<'a, T: Into<Quotable<'a>>>(source: T) -> Quotable<'a> {
        source.into()
    }

    into_quotable(bytes);
    into_quotable(byte_slice);
    into_quotable(&vec);
    into_quotable(&os_string);
    into_quotable(&b_string);
    into_quotable(&path_buf);
    into_quotable(&string);
}

#[test]
fn test_quote_ref_ext_byte_array() {
    let source = b"bytes!";
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'bytes!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'bytes!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'bytes!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'bytes!'"), quoted);
}

#[test]
fn test_quote_ref_ext_byte_slice() {
    let source = &b"bytes!"[..];
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'bytes!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'bytes!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'bytes!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'bytes!'"), quoted);
}

#[test]
fn test_quote_ref_ext_vec() {
    let source = Vec::from(b"vec!");
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'vec!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'vec!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'vec!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'vec!'"), quoted);
}

#[test]
fn test_quote_ref_ext_os_string() {
    let source = OsString::from("os-string!");
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'os-string!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'os-string!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'os-string!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'os-string!'"), quoted);
}

#[test]
fn test_quote_ref_ext_os_str() {
    let source = OsString::from("os-str!");
    let source = source.as_os_str();
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'os-str!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'os-str!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'os-str!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'os-str!'"), quoted);
}

#[test]
fn test_quote_ref_ext_b_string() {
    let source = bstr::BString::from(b"b-string!");
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'b-string!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'b-string!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'b-string!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'b-string!'"), quoted);
}

#[test]
fn test_quote_ref_ext_b_str() {
    let source = bstr::BString::from(b"b-str!");
    let source: &bstr::BStr = source.as_ref();
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'b-str!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'b-str!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'b-str!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'b-str!'"), quoted);
}

#[test]
fn test_quote_ref_ext_path_buf() {
    let source = std::path::PathBuf::from("path-buf!");
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'path-buf!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'path-buf!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'path-buf!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'path-buf!'"), quoted);
}

#[test]
fn test_quote_ref_ext_path() {
    let source = std::path::PathBuf::from("path!");
    let source = source.as_path();
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'path!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'path!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'path!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'path!'"), quoted);
}

#[test]
fn test_quote_ref_ext_string() {
    let source = "string!".to_owned();
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'string!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'string!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'string!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'string!'"), quoted);
}

#[test]
fn test_quote_ref_ext_str() {
    let source = "str!";
    let quoted: Vec<u8> = source.quoted(Bash);
    assert_eq!(Vec::from(b"$'str!'"), quoted);
    let quoted: OsString = source.quoted(Bash);
    assert_eq!(OsString::from_vec(b"$'str!'".into()), quoted);
    let quoted: BString = source.quoted(Bash);
    assert_eq!(BString::from(b"$'str!'"), quoted);
    let quoted: String = source.quoted(Bash);
    assert_eq!(String::from("$'str!'"), quoted);
}

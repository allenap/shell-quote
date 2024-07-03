/// Escape a byte as a 4-byte hex escape sequence.
///
/// The `\\xHH` format (backslash, a literal "x", two hex characters) is
/// understood by many shells.
#[inline]
#[cfg(feature = "bash")]
pub(crate) fn u8_to_hex_escape(ch: u8) -> [u8; 4] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        b'\\',
        b'x',
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

/// Escape a byte as a 4-byte hex escape sequence _with uppercase "X"_.
///
/// The `\\XHH` format (backslash, a literal "X", two hex characters) is
/// understood by fish. The `\\xHH` format is _also_ understood, but until fish
/// 3.6.0 it had a weirdness. From the [release notes][]:
///
/// > The `\\x` and `\\X` escape syntax is now equivalent. `\\xAB` previously
/// > behaved the same as `\\XAB`, except that it would error if the value “AB”
/// > was larger than “7f” (127 in decimal, the highest ASCII value).
///
/// [release notes]: https://github.com/fish-shell/fish-shell/releases/tag/3.6.0
///
#[inline]
#[cfg(feature = "fish")]
pub(crate) fn u8_to_hex_escape_uppercase_x(ch: u8) -> [u8; 4] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        b'\\',
        b'X',
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "bash")]
    fn test_u8_to_hex_escape() {
        for ch in u8::MIN..=u8::MAX {
            let expected = format!("\\x{ch:02X}");
            let observed = super::u8_to_hex_escape(ch);
            let observed = std::str::from_utf8(&observed).unwrap();
            assert_eq!(observed, &expected);
        }
    }

    #[test]
    #[cfg(feature = "fish")]
    fn test_u8_to_hex_escape_uppercase_x() {
        for ch in u8::MIN..=u8::MAX {
            let expected = format!("\\X{ch:02X}");
            let observed = super::u8_to_hex_escape_uppercase_x(ch);
            let observed = std::str::from_utf8(&observed).unwrap();
            assert_eq!(observed, &expected);
        }
    }
}

/// Represent a single byte as a 2-byte hex number.
#[allow(unused)]
pub(crate) fn u8_to_hex(ch: u8) -> [u8; 2] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

/// Escape a byte as a 4-byte hex escape sequence.
///
/// The `\\xHH` format (backslash, a literal "x", two hex characters) is
/// understood by many shells.
pub(crate) fn u8_to_hex_escape(ch: u8) -> [u8; 4] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        b'\\',
        b'x',
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_hex() {
        for ch in u8::MIN..=u8::MAX {
            let expected = format!("{ch:02X}");
            let observed = u8_to_hex(ch);
            let observed = std::str::from_utf8(&observed).unwrap();
            assert_eq!(observed, &expected);
        }
    }

    #[test]
    fn test_u8_to_hex_escape() {
        for ch in u8::MIN..=u8::MAX {
            let expected = format!("\\x{ch:02X}");
            let observed = u8_to_hex_escape(ch);
            let observed = std::str::from_utf8(&observed).unwrap();
            assert_eq!(observed, &expected);
        }
    }
}

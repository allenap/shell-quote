pub(crate) fn u8_to_hex(ch: u8) -> [u8; 2] {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    [
        HEX_DIGITS[(ch >> 4) as usize],
        HEX_DIGITS[(ch & 0xF) as usize],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_hex() {
        assert_eq!(u8_to_hex(0), [b'0', b'0']);
        assert_eq!(u8_to_hex(1), [b'0', b'1']);
        assert_eq!(u8_to_hex(10), [b'0', b'A']);
        assert_eq!(u8_to_hex(15), [b'0', b'F']);
        assert_eq!(u8_to_hex(16), [b'1', b'0']);
        assert_eq!(u8_to_hex(255), [b'F', b'F']);
        assert_eq!(u8_to_hex(b'a'), [b'6', b'1']);
    }
}

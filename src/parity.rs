/// Calculates the parity byte of an IBIS message. The given slice should
/// include the terminating CR, but the empty slice is also regarded as a
/// valid argument and returns 0x7F.
pub fn parity_byte(data: &[u8]) -> u8 {
    const EMPTY_PARITY: u8 = 0x7F;
    data.iter().fold(EMPTY_PARITY, |acc, next| acc ^ next)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_parity() {
        let parity = parity_byte(&[]);
        assert_eq!(
            parity, 0x7F,
            "Unexpected parity of empty message, got {:X?}",
            parity
        )
    }

    #[test]
    fn carriage_return_parity() {
        let parity = parity_byte(&[b'\r']);
        assert_eq!(
            parity, 0x72,
            "Unexpected checksum of message consisiting only of terminating CR, got: {:X?}",
            parity
        )
    }

    #[test]
    fn line_26_parity_byte() {
        assert_eq!(
            parity_byte(&[b'l', b'0', b'2', b'6', b'\r']),
            0x2A,
            "Unexpected result for parity byte of known telegram"
        )
    }
}

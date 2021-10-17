//! Types for IBIS telegrams.
//!
//! Not sure about the correct English terminology for the concept, so when
//! we speak of a _telegram_ in documentation, we mean the concept that is
//! called "Datensatz" in german, e.g. DS003.

use std::fmt;
use std::str::from_utf8;

/// A telegram in the IBIS protocol, binary, including trailing carriage return
/// and checksum. The contained data is guaranteed to be a valid telegram
/// that can be sent over the bus.
///
/// For example, [Telegram::destination(u8)][Telegram::destination(u8)]
/// produces the DS003 telegram.
pub struct Telegram(Vec<u8>);

impl fmt::Debug for Telegram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len_excl_cr_and_partiy = self.0.len() - 2;
        let human_readable_part = from_utf8(&self.0[0..len_excl_cr_and_partiy])
            // excluding trailer, telegrams produced through normal means are always valid strings
            .unwrap();
        let parity_byte = self.0[self.0.len() - 1];
        write!(
            f,
            "{str}<CR><P:{parity:X?}>",
            str = human_readable_part,
            parity = parity_byte
        )
    }
}

impl Telegram {
    /// Produces a DS001 telegram, selecting a line number.
    ///
    /// Number must be in range 1–999.
    ///
    /// # Panics
    /// If the destination is zero or can not be represented with
    /// three digits, that is, if greater than 999, then the function
    /// will panic.
    pub fn line(line_nr: u16) -> Telegram {
        assert!(
            line_nr > 0 && line_nr <= 999,
            "Line must be in range 1--999 so that it is non-zero and can be represented with three decimal digits"
          );
        let mut telegram = alloc_telegram(4); // l000 has four bytes
        telegram.push(b'l');
        append_three_digits(line_nr, &mut telegram);
        finish_telegram(telegram)
    }

    /// Produces a DS003 telegram, selecting a destination by index.
    ///
    /// Number must be in range 0–999.
    ///
    /// # Panics
    /// If the destination can not be represented with three digits,
    /// that is, if greater than 999, then this function panics.
    pub fn destination(destination_idx: u16) -> Telegram {
        assert!(
          destination_idx <= 999,
          "Destination must be in range 0--999 so that it can be represented with three decimal digits"
        );
        let mut telegram = alloc_telegram(4); // l000 has four bytes
        telegram.push(b'z');
        append_three_digits(destination_idx, &mut telegram);
        finish_telegram(telegram)
    }

    /// Produces a DS20 telegram, querying the status of a display device. Suitable for
    /// both interior or exterior displays.
    /// 
    /// Given address must be range 0-15 so that it can be represented with ASCII digits
    /// ranging from `b'0'` to `b'?'`. The standard does not seem to allow `b'0'` as an
    /// address but we have seen software in the wild that uses address 0 so we support
    /// it here.
    /// 
    /// The response is an `a<status>` followed by some ASCII decimal for the status.
    /// Statuses `b'0'` and `b'3'` have been seen in the wild, but their meaning is not
    /// clear.
    /// 
    /// # Panics
    /// This function panics if the address is higher than 15.
    pub fn display_status(address: u8) -> Telegram {
        assert!(address <= 15, "Address for display status query must be in range 0-15");
        let mut telegram = alloc_telegram(2);
        telegram.push(b'a');
        append_address(address, &mut telegram);
        finish_telegram(telegram)
    }

    /// Produces a DS120 telegram, querying the software version or versionf of a display
    /// device. Suitable for both interior or exterior displays.
    /// 
    /// Given address must be range 0-15 so that it can be represented with ASCII digits
    /// ranging from `b'0'` to `b'?'`. The standard does not seem to allow `b'0'` as an
    /// address but we have seen software in the wild that uses address 0 so we support
    /// it here.
    /// 
    /// Example response from a BS210 flipdot display: `aVV2.3RigaB/H7/99`.
    /// 
    /// # Panics
    /// This function panics if the address is higher than 15.
    pub fn display_version(address: u8) -> Telegram {
        assert!(address <= 15, "Address for display version query must be in range 0-15");
        let mut telegram = alloc_telegram(3);
        telegram.push(b'a');
        telegram.push(b'V');
        append_address(address, &mut telegram);
        finish_telegram(telegram)
    }

    /// Gets the telegram as an immutable sequence of bytes, including carriage return
    /// and paritiy byte.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

fn alloc_telegram(telegram_len: usize) -> Vec<u8> {
    // 2 extra bytes for CR and parity byte
    Vec::with_capacity(telegram_len + 2)
}

fn append_three_digits(num: u16, onto: &mut Vec<u8>) {
    assert!(num <= 999);

    let hundreds = num / 100;
    let tens = (num - hundreds * 100) / 10;
    let ones = num - hundreds * 100 - tens * 10;

    let hundreds = b'0' + (hundreds as u8);
    let tens = b'0' + (tens as u8);
    let ones = b'0' + (ones as u8);

    onto.push(hundreds);
    onto.push(tens);
    onto.push(ones);
}

fn append_address(address: u8, onto: &mut Vec<u8>) {
    assert!(address <= 15);
    let address = b'0' + address;
    onto.push(address);
}

fn parity_byte(data: &[u8]) -> u8 {
    const EMPTY_PARITY: u8 = 0x7F;
    data.iter().fold(EMPTY_PARITY, |acc, next| acc ^ next)
}

/// Appends the final CR and parity byte and returns the finished telegram.
///
/// Typically does not allocate for fixed-length telegrams, which have been
/// allocated with `alloc_telegram`.
fn finish_telegram(mut content: Vec<u8>) -> Telegram {
    content.push(b'\r'); // parity includes carriage return
    content.push(parity_byte(&content));
    Telegram(content)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn line_26() {
        let telegram = Telegram::line(26);
        assert_eq!(telegram.as_bytes(), &[b'l', b'0', b'2', b'6', b'\r', 0x2A])
    }

    #[test]
    #[should_panic]
    fn line_0() {
        Telegram::line(0);
    }

    #[test]
    #[should_panic]
    fn line_1000() {
        Telegram::line(1000);
    }

    #[test]
    #[should_panic]
    fn line_umax() {
        Telegram::line(std::u16::MAX);
    }

    #[test]
    fn destination_0() {
        let telegram = Telegram::destination(0);
        assert_eq!(
            telegram.as_bytes(),
            &[
                b'z',
                b'0',
                b'0',
                b'0',
                b'\r',
                0x7F ^ b'z' ^ b'0' ^ b'0' ^ b'0' ^ b'\r'
            ]
        )
    }

    #[test]
    fn destination_1() {
        let telegram = Telegram::destination(1);
        assert_eq!(
            telegram.as_bytes(),
            &[
                b'z',
                b'0',
                b'0',
                b'1',
                b'\r',
                0x7F ^ b'z' ^ b'0' ^ b'0' ^ b'1' ^ b'\r'
            ]
        )
    }

    #[test]
    fn destination_31() {
        let telegram = Telegram::destination(31);
        assert_eq!(
            telegram.as_bytes(),
            &[
                b'z',
                b'0',
                b'3',
                b'1',
                b'\r',
                0x7F ^ b'z' ^ b'0' ^ b'3' ^ b'1' ^ b'\r'
            ]
        )
    }

    #[test]
    fn destination_938() {
        let telegram = Telegram::destination(938);
        assert_eq!(
            telegram.as_bytes(),
            &[
                b'z',
                b'9',
                b'3',
                b'8',
                b'\r',
                0x7F ^ b'z' ^ b'9' ^ b'3' ^ b'8' ^ b'\r'
            ]
        )
    }

    #[test]
    #[should_panic]
    fn destination_1000() {
        Telegram::destination(1000);
    }

    #[test]
    #[should_panic]
    fn destination_umax() {
        Telegram::destination(std::u16::MAX);
    }

    #[test]
    fn destination_523_debug_repr() {
        let telegram = Telegram::destination(523);
        let telegram = &format!("{:?}", telegram);
        assert_eq!(telegram, "z523<CR><P:3C>");
    }

    #[should_panic]
    #[test]
    fn version_16_panics() {
        Telegram::display_version(16);
    }

    #[should_panic]
    #[test]
    fn status_17_panics() {
        Telegram::display_status(16);
    }

    #[test]
    fn display_version_one() {
        let telegram = Telegram::display_version(1);
        let telegram = &format!("{:?}", telegram);
        assert_eq!(telegram, "aV1<CR><P:74>");
    }

    #[test]
    fn display_status_questionmark() {
        let telegram = Telegram::display_status(15);
        let telegram = &format!("{:?}", telegram);
        assert_eq!(telegram, "a?<CR><P:2C>");
    }

    #[test]
    fn display_status_zero() {
        let telegram = Telegram::display_status(0);
        let telegram = &format!("{:?}", telegram);
        assert_eq!(telegram, "a0<CR><P:23>");
    }
}

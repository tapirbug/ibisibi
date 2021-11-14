//! Types that generate IBIS telegrams.
//!
//! Not sure about the correct English terminology for the concept, so when
//! we speak of a _telegram_ in documentation, we mean the concept that is
//! called "Datensatz" in german, e.g. DS003.

use builder::Builder;
use std::{
    fmt,
    str::from_utf8
};

pub use parse::TelegramParseError;

/// A telegram in the IBIS protocol, binary, including trailing carriage return
/// and checksum. The contained data is guaranteed to be a valid telegram
/// that can be sent over the bus or that has been received over the bus and is
/// valid.
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
        Builder::with_msg_len(4) // l000 has four bytes
            .byte(b'l')
            .three_digits(line_nr)
            .finish()
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
        Builder::with_msg_len(4) // z000 has four bytes
            .byte(b'z')
            .three_digits(destination_idx)
            .finish()
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
        assert!(
            address <= 15,
            "Address for display status query must be in range 0-15"
        );
        Builder::with_msg_len(2) // a0 has two bytes
            .byte(b'a')
            .address(address)
            .finish()
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
        assert!(
            address <= 15,
            "Address for display version query must be in range 0-15"
        );
        Builder::with_msg_len(3) // aV0 has three bytes
            .byte(b'a')
            .byte(b'V')
            .address(address)
            .finish()
    }

    /// Command of unknown purpose that is sent before flashing a sign database to a
    /// BS210 sign on specific address. It has a prefix of 0D 72 before the actual message
    /// that is not included in the checksum.
    ///
    /// In our tests we never saw any response to this message, so it might also not be
    /// relevant at all.
    pub fn bs_select_address(address: u8) -> Telegram {
        assert!(
            address <= 15,
            "Address for select address must be in range 0-15"
        );
        Builder::with_msg_len(5)
            .prefix(&[0x0D, 0x72])
            .byte(0x1B)
            .byte(b'S')
            .address(address)
            .finish()
    }

    /// Gets the telegram payload, that is, the part before CR and the checksum.
    /// If the message has a prefix, that will be included in the returned payload.
    pub fn payload(&self) -> &[u8] {
        &self.0[..self.0.len() - 2]
    }

    /// Gets the telegram as an immutable sequence of bytes, including carriage return
    /// and parity byte.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

mod builder {
    use super::Telegram;
    use crate::parity::parity_byte;

    pub struct Builder {
        prefix_len: usize,
        message: Vec<u8>,
    }

    impl Builder {
        pub fn with_msg_len(expected_len: usize) -> Self {
            Builder {
                prefix_len: 0,
                // 2 extra bytes for CR and parity byte
                message: Vec::with_capacity(expected_len + 2),
            }
        }

        /// Adds a prefix at the start of the message that is not included in the checksum.
        pub fn prefix(mut self, prefix: &[u8]) -> Self {
            assert!(
                self.message.is_empty(),
                "expected empty message when specifying prefix"
            );
            self.prefix_len = prefix.len();
            self.message.extend(prefix);
            self
        }

        pub fn byte(mut self, byte: u8) -> Self {
            self.message.push(byte);
            self
        }

        pub fn digit(self, digit: u8) -> Self {
            assert!(digit < 10, "digit out of range 0..=9");
            let digit = b'0' + digit;
            self.byte(digit)
        }

        pub fn address(self, address: u8) -> Self {
            assert!(address < 16, "address out of range 0..=15");
            let address = b'0' + address;
            self.byte(address)
        }

        pub fn three_digits(self, num: u16) -> Self {
            assert!(num <= 999, "digits out of range 0..=999");
            let hundreds = num / 100;
            let tens = (num - hundreds * 100) / 10;
            let ones = num - hundreds * 100 - tens * 10;
            self.digit(hundreds as u8)
                .digit(tens as u8)
                .digit(ones as u8)
        }

        /// Appends the final CR and parity byte and returns the finished telegram.
        pub fn finish(mut self) -> Telegram {
            // parity includes carriage return
            self.message.push(b'\r');
            // prefix_len is always <= message len when constructed through Builder methods
            let parity = parity_byte(&self.message[self.prefix_len..]);
            self.message.push(parity);
            // take message and leave empty message in the builder
            Telegram(self.message)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn build_status() {
            let telegram = Builder::with_msg_len(2).byte(b'a').digit(0).finish().0;
            assert_eq!(telegram, vec![b'a', b'0', b'\r', 0x23])
        }
    }
}

mod parse {
    use crate::parity::parity_byte;
    use super::Telegram;
    use std::convert::TryFrom;
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum TelegramParseError {
        #[error("Received unexpected IBIS response checksum {received}, expected {expected}")]
        Parity {
            expected: u8,
            received :u8
        },
        #[error("Received IBIS response that does not conform to the expected format with carriage return and parity byte")]
        Malformed
    }

    impl<'a> TryFrom<&'a [u8]> for Telegram {
        type Error = TelegramParseError;

        fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
            if buf.len() < 2 || buf[buf.len() - 2] != b'\r' {
                return Err(TelegramParseError::Malformed);
            }

            let expected_parity = parity_byte(&buf[..buf.len() - 1]);
            let received_parity = buf[buf.len() - 1];

            if expected_parity != received_parity {
                return Err(TelegramParseError::Parity {
                    expected: expected_parity,
                    received: received_parity
                });
            }

            Ok(Telegram(buf.into()))
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use std::convert::TryInto;

        #[test]
        fn status_query_response() {
            const RECEIVED : &[u8] = &[
                0x61, 0x30, 0x0d, 0x23
            ];
            let parsed : Telegram = RECEIVED.try_into().unwrap();
            assert_eq!(
                &parsed.0[..],
                RECEIVED
            );
        }

        #[test]
        fn version_query_response() {
            const RECEIVED : &[u8] = &[
                0x61, 0x56, 0x56, 0x32, 0x2e, 0x33, 0x52, 0x69, 0x67, 0x61, 0x42, 0x2f, 0x48, 0x37, 0x2f, 0x39, 0x39, 0x0d, 0x3c
            ];
            let parsed : Telegram = RECEIVED.try_into().unwrap();
            assert_eq!(
                &parsed.0[..],
                RECEIVED
            );
        }

        #[test]
        fn empty() {
            const RECEIVED : &[u8] = &[];
            let error : TelegramParseError = Telegram::try_from(RECEIVED).unwrap_err();
            assert_eq!(
                error,
                TelegramParseError::Malformed
            );
        }

        #[test]
        fn empty_payload_with_missing_checksum() {
            const RECEIVED : &[u8] = b"\r";
            let error : TelegramParseError = Telegram::try_from(RECEIVED).unwrap_err();
            assert_eq!(
                error,
                TelegramParseError::Malformed
            );
        }

        #[test]
        fn empty_payload_with_incorrect_checksum() {
            const RECEIVED : &[u8] = &[ b'\r', 0x42 ];
            let error : TelegramParseError = Telegram::try_from(RECEIVED).unwrap_err();
            assert_eq!(
                error,
                TelegramParseError::Parity {
                    expected: 0x7F ^ b'\r',
                    received: 0x42
                }
            );
        }

        #[test]
        fn non_empty_payload_with_incorrect_checksum() {
            const RECEIVED : &[u8] = &[ 0x11, b'\r', 0x42 ];
            let error : TelegramParseError = Telegram::try_from(RECEIVED).unwrap_err();
            assert_eq!(
                error,
                TelegramParseError::Parity {
                    expected: 0x7F ^ 0x11 ^ b'\r',
                    received: 0x42
                }
            );
        }
    }
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
            telegram.payload(),
            b"z000"
        );
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
            telegram.payload(),
            b"z001"
        );
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
            telegram.payload(),
            b"z031"
        );
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
            telegram.payload(),
            b"z938"
        );
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
    fn display_version_of_address_one() {
        let telegram = Telegram::display_version(1);
        assert_eq!(
            telegram.payload(),
            b"aV1"
        );
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
        assert_eq!(
            telegram.payload(),
            b"a0"
        );
        let telegram = &format!("{:?}", telegram);
        assert_eq!(telegram, "a0<CR><P:23>");
    }

    #[test]
    fn select_address_1() {
        let telegram = Telegram::bs_select_address(1);
        assert_eq!(
            telegram.payload(),
            &[0x0d, 0x72, 0x1b, 0x53, 0x31],
            "Payload of messages with prefix should include the prefix"
        );
        assert_eq!(
            telegram.as_bytes(),
            &[0x0d, 0x72, 0x1b, 0x53, 0x31, 0x0d, 0x0b]
        );
    }
}

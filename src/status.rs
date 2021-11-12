use crate::{
    serial::Serial,
    telegram::Telegram,
    parity::parity_byte
};
use thiserror::Error;
use std::io::{Read, Write};
use std::fmt::{self, Formatter, Display};

pub type Result<T> = std::result::Result<T, Error>;

pub fn status(serial: &mut Serial, address: u8) -> Result<Status> {
    assert!(address < 16, "Expected address in range 0..=15");

    let telegram = Telegram::display_status(address);
    serial.write(telegram.as_bytes())?;

    let mut response = [0_u8; 4];
    serial.read_exact(&mut response)?;

    let received_checksum = response[3];
    let expected_checksum = parity_byte(&response[0..3]);
    if received_checksum != expected_checksum {
        return Err(Error::Parity {
            expected: expected_checksum,
            got: received_checksum
        })
    }

    let status_char = response[1];
    let status = status_char.into();
    Ok(status)
}

/// Responses from the display status command. Not well understood.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// When listing devices in the beginning, we always got back status
    /// `b'3'` with unknown meaning, but presumably that everything is ok.
    Ok,
    /// The status with number `b'0'` has been seen once before clearing and
    /// flashing. The meaning is not really clear but it does not seem to
    /// prohibit starting to flash so we called it `ReadyForData` for now.
    ReadyForData,
    /// The meanings of most numbers are not clear. Everything other than
    /// `b'0'` and `b'3'` gets constructed as this variant. The `u8` holds the
    /// value of the unknown status as sent over the wire, so presumably
    /// `b'1'`, `b'2'`, or `b'4'` onwards, that is, the ASCII digit is not
    /// converted to a number.
    Uncategorized(u8)
}

impl From<u8> for Status {
    fn from(status_byte: u8) -> Self {
        match status_byte {
            b'0' => Status::ReadyForData,
            b'3' => Status::Ok,
            other => Status::Uncategorized(other)
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "Ok (3)"),
            Status::ReadyForData => write!(f, "Ready for data (0)"),
            Status::Uncategorized(status) => write!(f, "Unknown status ({})", status),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not query display status due to serial port error: {0}")]
    IO(#[from] std::io::Error),
    #[error("corrupt message, found parity byte {got}, expecting {expected}")]
    Parity { expected: u8, got: u8 },
}

impl Error {
    pub fn is_timed_out(&self) -> bool {
        match self {
            Error::IO(err) if err.kind() == std::io::ErrorKind::TimedOut => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn timeout() {
        let mut serial = Serial::builder()
                .time_out()
                .build();

        let err = status(&mut serial, 0).unwrap_err();

        assert!(
            err.is_timed_out(),
            "Expected timeout error"
        )
    }

    #[test]
    fn checksum_err() {
        let mut serial = Serial::builder()
                .receive(b"a0\r0") // correct checksum would be #, not 0
                .build();

        let err = status(&mut serial, 0).unwrap_err();

        match err {
            Error::Parity { .. } => {}
            err => panic!("Unexpected error: {:?}", err)
        }
    }

    #[test]
    fn ok() {
        let mut serial = Serial::builder()
                .receive(b"a3\r ")
                .build();

        let status = status(&mut serial, 0).unwrap();

        assert_eq!(
            status,
            Status::Ok,
            "Expected status 3 to be interpreted as Ok"
        )
    }

    #[test]
    fn ready_for_data() {
        let mut serial = Serial::builder()
                .receive(b"a0\r#")
                .build();

        let status = status(&mut serial, 9).unwrap();

        assert_eq!(
            status,
            Status::ReadyForData,
            "Expected status 0 to be interpreted as ReadyForData"
        )
    }

    #[test]
    fn uncategorized_status() {
        let mut serial = Serial::builder()
                .receive(b"a7\r$")
                .build();

        let status = status(&mut serial, 8).unwrap();

        assert_eq!(
            status,
            Status::Uncategorized(b'7'),
            "Expected status 7 to be uncategorized"
        )
    }

    #[should_panic]
    #[test]
    fn address_out_of_bounds() {
        let mut serial = Serial::builder().build();
        status(&mut serial, 0x10).unwrap();
    }
}

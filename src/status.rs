use crate::{
    serial::Serial,
    telegram::Telegram,
    parity::parity_byte
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Queries the status of the display device at the given index and
/// returns the response as a number starting at 0.
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
    /// `b'1'`, `b'2'`, or `b'4'` onwards.
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

}

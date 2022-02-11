use super::{checksum::checksum, Error, Record, Result};
use std::mem::take;

pub struct Builder {
    data: Vec<u8>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            data: vec![
                0x00, // reserve this byte for the length, but set it to zero for now
            ],
        }
    }

    /// Appends a single byte to the record.
    pub fn u8(&mut self, data: u8) -> &mut Self {
        self.data.push(data);
        self
    }

    /// Appends a number in little-endian byte order to the record.
    pub fn u16(&mut self, data: u16) -> &mut Self {
        self.buf(&data.to_le_bytes())
    }

    /// Appends a buffer to the record.
    pub fn buf(&mut self, data: &[u8]) -> &mut Self {
        self.data.extend(data);
        self
    }

    /// Finishes the build, consuming the contents and leaving a fresh builder in place.
    pub fn build(&mut self) -> Result<Record> {
        let payload_len = if self.data.len() <= 0x100 {
            (self.data.len() - 1) as u8
        } else {
            // Length (excluding length/checksum bytes) must fit in a single byte
            return Err(Error::RecordLengthOutOfBounds);
        };

        let mut data = take(&mut self.data);
        data[0] = payload_len; // first byte is length, excluding both length byte itself and checksum
        data.push(checksum(&data));

        debug_assert!(
            data.len() >= 2,
            "When constructed through new, assumed that the length is always 2 or more"
        );
        let record = Record { data };
        Ok(record)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buf() {
        const BUF_EXPECTED_RESULT: &[u8] = &[
            0x24, 0x05, 0x00, 0x00, 0x00, 0x57, 0x00, 0x12, 0x00, 0x1b, 0x00, 0x12, 0x1c, 0x8b,
            0x45, 0x06, 0xf9, 0x00, 0xe0, 0x01, 0x00, 0x0a, 0xe0, 0x01, 0x05, 0x0a, 0x00, 0x80,
            0x01, 0x60, 0x01, 0xa0, 0x00, 0x4f, 0x00, 0x00, 0x30, 0x7a,
        ];
        let buf_contents = &BUF_EXPECTED_RESULT[1..(BUF_EXPECTED_RESULT.len() - 1)];

        let record = Builder::new().buf(buf_contents).build().unwrap();

        assert_eq!(record.as_bytes(), BUF_EXPECTED_RESULT);
        assert_eq!(record.payload(), buf_contents);
        assert_eq!(
            record.checksum(),
            BUF_EXPECTED_RESULT[BUF_EXPECTED_RESULT.len() - 1]
        );
    }
}

use super::{checksum::checksum, Error, Record, Result};
use std::mem::take;

pub struct Builder {
    data: Vec<u8>,
    /// Position of the message being built.
    ///
    /// If non-zero, this is a builder for a multi-message record and the first
    /// message is already finished.
    build_idx: usize,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            data: vec![
                0x00, // reserve this byte for the length, but set it to zero for now
            ],
            build_idx: 0,
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

    fn set_msg_len(&mut self) -> Result<&mut Self> {
        let msg = &mut self.data[self.build_idx..];
        debug_assert!(
            !msg.is_empty(),
            "Expected at least the length byte placeholder to be present"
        );

        let payload_len = msg.len() - 1;
        let payload_len = if payload_len >= 0x100 {
            // Length (excluding length/checksum bytes) must fit in a single byte
            return Err(Error::RecordLengthOutOfBounds);
        } else {
            payload_len as u8
        };
        // first byte is length, excluding both length byte itself and checksum
        msg[0] = payload_len;
        Ok(self)
    }

    // Adds the checksum, assuming that set_msg_len has already been called.
    fn push_checksum(&mut self) -> &mut Self {
        debug_assert!(
            (self.data.len() - self.build_idx) >= 1,
            "Expected at least the length to be present"
        );
        let checksum = checksum(&self.data[self.build_idx..]); // calculate checksum including length
        self.data.push(checksum);
        self
    }

    fn finish_msg(&mut self) -> Result<&mut Self> {
        self.set_msg_len()?;
        Ok(self.push_checksum())
    }

    /// Finishes this message and starts a new one in this record.
    #[cfg(test)]
    pub fn start_next(&mut self) -> Result<&mut Self> {
        self.finish_msg()?;

        // set next build idx and reserve a byte for the length of the next message
        self.build_idx = self.data.len();
        self.data.push(0x00); // byte is reserved for length of next message, set to zero for now

        Ok(self)
    }

    /// Finishes the build, consuming the contents and leaving an empty builder in place.
    pub fn build(&mut self) -> Result<Record> {
        self.finish_msg()?;
        let data = take(&mut self.data);
        self.build_idx = 0;
        debug_assert!(
            data.len() >= 2,
            "When constructed through builder, assumed that the length is always 2 or more (length + checksum bytes and optional content)"
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

    #[test]
    fn build_multi_msg() {
        let built = Builder::new()
            .u8(0x0f)
            .start_next()
            .unwrap()
            .u8(0x0f)
            .start_next()
            .unwrap()
            .u8(0x0f)
            .start_next()
            .unwrap()
            .u8(0x0f)
            .build()
            .unwrap();
        assert_eq!(
            built.as_bytes(),
            &[0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0],
            "Unexpected byte pattern for multi-message sent after flashing has finished."
        );
    }
}

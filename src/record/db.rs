use super::{Builder, Error, Record, Result};

/// A record that represents a chunk from the line database, on the granularity of
/// a single IHEX record, which can be sent over the wire for flashing of a flipdot
/// display.
pub struct DatabaseChunk(Record);

impl DatabaseChunk {
    /// Creates a chunk of the sign database to be sent over the wire to the BS210
    /// sign, with the given content data written to the given address. The address
    /// is specified in native endianness.
    pub fn new(address: u16, content: &[u8]) -> Result<Self> {
        if content.len() > 0xFF {
            return Err(Error::RecordLengthOutOfBounds);
        }

        Builder::new()
            // 1 byte 5 (unknown purpose)
            .u8(0x05)
            // 2 bytes address (little endian)
            .u16(address)
            // record type 0 (data record)
            .u8(0)
            // then the actual database content
            .buf(content)
            .build()
            .map(DatabaseChunk)
    }

    /// The bytes of the full record, including the length and the checksum.
    ///
    /// Guaranteed to have a size of four bytes or more.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    #[cfg(test)]
    pub fn address(&self) -> u16 {
        let payload = self.0.payload();
        u16::from_le_bytes([payload[1], payload[2]])
    }

    /// The data part of the record.
    #[cfg(test)]
    pub fn data(&self) -> &[u8] {
        &self.0.payload()[4..]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mini0_firstrecord() {
        // The raw data part of the first record in mini0.hex
        const DATA: &[u8] = &[
            0x57, 0x00, 0x12, 0x00, 0x1B, 0x00, 0x12, 0x1C, 0x8B, 0x45, 0x06, 0xF9, 0x00, 0xE0,
            0x01, 0x00, 0x0A, 0xE0, 0x01, 0x05, 0x0A, 0x00, 0x80, 0x01, 0x60, 0x01, 0xA0, 0x00,
            0x4F, 0x00, 0x00, 0x30,
        ];

        let chunk = DatabaseChunk::new(0, DATA).unwrap();

        assert_eq!(
            chunk.0.as_bytes()[0],
            0x24,
            "Unexpected length of resulting data part"
        );
        assert_eq!(chunk.address(), 0x0, "Unexpected address");
        assert_eq!(chunk.0.as_bytes()[2], 0x0, "Unexpected address");
        assert_eq!(chunk.0.as_bytes()[3], 0x0, "Unexpected address");
        assert_eq!(chunk.data(), DATA, "Unexpected content data");
        assert_eq!(
            chunk.0.checksum(),
            0x7A,
            "Expected checksum: 7A but got {:X?}",
            chunk.0.checksum()
        )
    }

    #[test]
    fn mini0_first_record_with_two_byte_address() {
        // The raw data part of the first record in mini0.hex
        const DATA: &[u8] = &[0; 32];

        let record = DatabaseChunk::new(0x0100, DATA).unwrap();

        assert_eq!(
            record.0.payload().len(),
            0x24,
            "Unexpected length of resulting data part"
        );
        assert_eq!(
            record.0.payload().len(),
            record.0.as_bytes()[0] as usize,
            "Expected length to be first byte"
        );
        assert_eq!(record.address(), 0x0100, "Unexpected address");
        assert_eq!(record.0.as_bytes()[2], 0x00, "Unexpected address");
        assert_eq!(record.0.as_bytes()[3], 0x01, "Unexpected address");
        assert_eq!(record.data(), DATA, "Unexpected content data");
        assert_eq!(
            record.0.checksum(),
            0xD6,
            "Expected checksum: D6 but got {:X?}",
            record.0.checksum()
        )
    }
}

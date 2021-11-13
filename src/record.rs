pub type Result<T> = std::result::Result<T, Error>;
pub use db::DatabaseChunk;

use builder::Builder;

/// A chunk of a record database that corresponds to a single line in a sign database
/// of a BS210 sign, but in a different format that is understood by the sign when
/// sending the data over the wire, rather then when viewing it in an editor.
///
/// Also used for clearing the device and for querying some version information.
///
/// Contents are guaranteed to be 2 bytes or longer.
struct Record {
    data: Vec<u8>,
}

impl Record {
    /// Record data excluding the first (length) and last (checksum) bytes.
    pub fn payload(&self) -> &[u8] {
        &self.data[1..self.data.len()-1]
    }

    /// The bytes of the full record, including the length and the checksum.
    ///
    /// Guaranteed to have a size of two bytes or more.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn checksum(&self) -> u8 {
        self.data[self.data.len() - 1]
    }
}

fn calculate_checksum(data: &[u8]) -> u8 {
    !data.iter().cloned().fold(0, u8::wrapping_add) + 1
}

#[derive(Debug)]
pub enum Error {
    RecordLengthOutOfBounds,
}

mod db {
    use super::{Record, Result, Error, Builder};

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

        pub fn address(&self) -> u16 {
            let payload = self.0.payload();
            u16::from_le_bytes([payload[1], payload[2]])
        }

        /// The data part of the record.
        pub fn data(&self) -> &[u8] {
            &self.0.payload()[4..]
        }

        /// The bytes of the full record, including the length and the checksum.
        ///
        /// Guaranteed to have a size of four bytes or more.
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_bytes()
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
}

mod query {
    //! Query messages that are sent to obtain information from the sign.
    //!
    //! It is not known if it is necessary to make these queries to start the flashing
    //! process, but we do it in any case because it also verifies that what we are
    //! talking to behaves like a BS210 sign.

    use super::Record;

    /// First record to be sent after selecting the address.
    ///
    /// Device should send back `4f` when sending this query.
    ///
    /// It is not known what the query or the response actually mean.
    pub fn unknown_query_0() -> &'static Record {
        const PREPARE_FLASHING_0: &Record = &Record {
            data: vec![ 0x06, 0x01, 0x21, 0x00, 0x00, 0x00, 0x00, 0xd8 ]
        };
        PREPARE_FLASHING_0
    }

    /// Second record to be sent after `prepare_flashing_0`.
    ///
    /// Device should send back `4f 01 57 a8` when sending this query.
    ///
    /// It is not known what the query or the response actually mean.
    pub fn unknown_query_1() -> &'static Record {
        const PREPARE_FLASHING_1: &Record = &Record {
            data: vec![ 0x04, 0x08, 0x00, 0x20, 0x01, 0xd3 ]
        };
        PREPARE_FLASHING_1
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use super::super::Builder;

        /// Since length and checksum are handcoded, we need to make sure that
        /// we wrote everything down correctly by calculating a checksum over
        /// the buffer.
        #[test]
        fn unknown_query_0_integrity() {
            let static_unkown_query = unknown_query_0();
            let calculated_unknown_query = Builder::new()
                .buf(static_unkown_query.payload())
                .build()
                .unwrap();

            assert_eq!(
                static_unkown_query.checksum(),
                calculated_unknown_query.checksum()
            );
            assert_eq!(
                static_unkown_query.as_bytes(),
                calculated_unknown_query.as_bytes()
            );
        }

        #[test]
        fn unknown_query_1_integrity() {
            let static_unkown_query = unknown_query_1();
            let calculated_unknown_query = Builder::new()
                .buf(static_unkown_query.payload())
                .build()
                .unwrap();

            assert_eq!(
                static_unkown_query.checksum(),
                calculated_unknown_query.checksum()
            );
            assert_eq!(
                static_unkown_query.as_bytes(),
                calculated_unknown_query.as_bytes()
            );
        }
    }
}

mod builder {
    use std::mem::take;
    use super::{Record, calculate_checksum, Result, Error};

    pub struct Builder {
        data: Vec<u8>
    }

    impl Builder {
        pub fn new() -> Self {
            Builder {
                data: vec![
                    0x00 // reserve this byte for the length, but set it to zero for now
                ]
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
            data.push(calculate_checksum(&data));

            debug_assert!(data.len() >= 2, "When constructed through new, assumed that the length is always 2 or more");
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
                0x24, 0x05, 0x00, 0x00, 0x00, 0x57, 0x00, 0x12, 0x00, 0x1b, 0x00, 0x12, 0x1c, 0x8b, 0x45, 0x06,
                0xf9, 0x00, 0xe0, 0x01, 0x00, 0x0a, 0xe0, 0x01, 0x05, 0x0a, 0x00, 0x80, 0x01, 0x60, 0x01, 0xa0,
                0x00, 0x4f, 0x00, 0x00, 0x30, 0x7a
            ];
            let buf_contents = &BUF_EXPECTED_RESULT[1..(BUF_EXPECTED_RESULT.len() - 1)];

            let record = Builder::new().buf(buf_contents).build().unwrap();

            assert_eq!(
                record.as_bytes(),
                BUF_EXPECTED_RESULT
            );
            assert_eq!(
                record.payload(),
                buf_contents
            );
            assert_eq!(
                record.checksum(),
                BUF_EXPECTED_RESULT[BUF_EXPECTED_RESULT.len() - 1]
            );
        }
    }
}

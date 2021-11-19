pub type Result<T> = std::result::Result<T, Error>;
pub use db::DatabaseChunk;

use thiserror::Error;
use builder::Builder;

/// A chunk of a record database that corresponds to a single line in a sign database
/// of a BS210 sign, but in a different format that is understood by the sign when
/// sending the data over the wire, rather then when viewing it in an editor.
///
/// Also used for clearing the device and for querying some version information.
///
/// Contents are guaranteed to be 2 bytes or longer.
pub struct Record {
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

pub mod res {
    use super::{Result, Error, checksum};

    /// Verifies that a reponse from a BS210 conforms to the normal structure of a response
    /// received from BS210, that is, it starts with 0x4f, followed by a record.
    ///
    /// This method is to be used where the contents are not well-understood and only the
    /// length and checksum should be verified.
    pub fn verify_response_record(buf: &[u8]) -> Result<()> {
        response_payload(buf)?;
        Ok(())
    }

    /// Verifies that the given buffer holds an acknowledgement response without an attached
    /// record, that is 0x4F.
    pub fn verify_ack_response(buf: &[u8]) -> Result<()> {
        if buf.len() == 0 || buf[0] != 0x4f {
            return Err(Error::ResponseMagicNumberMissing);
        }

        if buf != &[ 0x4f ] {
            return Err(Error::ResponseNotAcknowledgement);
        }

        Ok(())
    }

    /// Verifies response integrity and returns the payload part (without len and checksum),
    /// if successful
    pub fn response_payload(buf: &[u8]) -> Result<&[u8]> {
        if buf.len() == 0 || buf[0] != 0x4f {
            return Err(Error::ResponseMagicNumberMissing);
        }
        let buf = &buf[1..];
        if buf.len() < 2 {
            return Err(Error::ResponseHeaderOrTrailerMissing);
        }

        let received_checksum = buf[buf.len() - 1];
        let buf = &buf[..buf.len() - 1];
        let expected_checksum = checksum(buf);
        if received_checksum != expected_checksum {
            return Err(Error::ResponseChecksumMismatch {
                expected: expected_checksum,
                received: received_checksum
            });
        }

        let buf_payload_len = buf[0];
        let buf = &buf[1..];
        if buf.len() > 0xFF {
            return Err(Error::ResponseRecordLengthOutOfBounds {
                len: buf.len()
            });
        }
        let received_payload_len = buf.len() as u8;
        if received_payload_len != buf_payload_len {
            return Err(Error::ResponsePayloadLenMismatch {
                expected: buf_payload_len,
                received: received_payload_len
            })
        }

        Ok(buf)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn ok_unknown_query_0_response() {
            const RESPONSE: &[u8] = &[ 0x4f, 0x01, 0x57, 0xa8 ];
            verify_response_record(RESPONSE).unwrap();
        }

        #[test]
        fn ok_other_response_of_unknown_purpose() {
            const RESPONSE: &[u8] = & [ 0x4f, 0x10, 0x00, 0x00, 0x02, 0x00, 0xdf, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xf7, 0xf7, 0x26 ];
            verify_response_record(RESPONSE).unwrap();
        }

        #[test]
        fn ok_panel_v_3_11_response() {
            const RESPONSE: &[u8] = &[
                0x4f, 0x10, 0x50, 0x41, 0x4e, 0x45, 0x4c, 0x20, 0x56, 0x33, 0x2e, 0x31, 0x31, 0x20, 0x20, 0x20, 0x20, 0x20, 0xa7
            ];
            verify_response_record(RESPONSE).unwrap();
        }

        #[test]
        fn checksum_missing_unknown_query_0_response() {
            const RESPONSE: &[u8] = &[ 0x4f, 0x01, 0x57 ];
            assert_eq!(
                verify_response_record(RESPONSE).unwrap_err(),
                Error::ResponseChecksumMismatch { expected: 0xFF, received: 0x57 }
            )
        }

        #[test]
        fn checksum_failure_unknown_query_0_response() {
            const RESPONSE: &[u8] = &[ 0x4f, 0x01, 0x57, 0xb9 ];
            assert_eq!(
                verify_response_record(RESPONSE).unwrap_err(),
                Error::ResponseChecksumMismatch { expected: 0xa8, received: 0xb9 }
            )
        }

        #[test]
        fn ok_ack() {
            const RESPONSE: &[u8] = &[ 0x4f ];
            verify_ack_response(RESPONSE).unwrap();
        }

        #[test]
        fn empty_ack() {
            assert_eq!(
                verify_ack_response(&[]).unwrap_err(),
                Error::ResponseMagicNumberMissing
            )
        }

        #[test]
        fn corrupt_ack() {
            const RESPONSE: &[u8] = &[ 0x5f ];
            assert_eq!(
                verify_ack_response(RESPONSE).unwrap_err(),
                Error::ResponseMagicNumberMissing
            )
        }

        #[test]
        fn ack_with_extra_bytes() {
            const RESPONSE: &[u8] = &[ 0x4f, 0x00 ];
            assert_eq!(
                verify_ack_response(RESPONSE).unwrap_err(),
                Error::ResponseNotAcknowledgement
            )
        }

        #[test]
        fn empty_response() {
            assert_eq!(
                verify_response_record(&[]).unwrap_err(),
                Error::ResponseMagicNumberMissing
            )
        }
    }
}

fn checksum(data: &[u8]) -> u8 {
    (!data.iter().cloned().fold(0, u8::wrapping_add)).wrapping_add(1)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Record length out of bounds")]
    RecordLengthOutOfBounds,
    #[error("Response from sign has length that is out of bounds: {len}")]
    ResponseRecordLengthOutOfBounds {
        len: usize
    },
    #[error("Response from sign corrupt, lacking magic number")]
    ResponseMagicNumberMissing,
    /// Expected a response holding just the magic number, but got a complex response.
    #[error("Acknowledgement expected but got complex response from sign")]
    ResponseNotAcknowledgement,
    #[error("Response from sign is too short, missing header, trailer, or both")]
    ResponseHeaderOrTrailerMissing,
    #[error("Response from sign corrupt, expected record length: {expected:X?}, got: {received:X?}")]
    ResponsePayloadLenMismatch {
        expected: u8,
        received: u8
    },
    #[error("Response from sign corrupt, expected checksum: {expected:X?}, got: {received:X?}")]
    ResponseChecksumMismatch {
        expected: u8,
        received: u8
    }
}

pub mod db {
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

pub mod query {
    //! Query messages that are sent to obtain information from the sign.
    //!
    //! It is not known if it is necessary to make these queries to start the flashing
    //! process, but we do it in any case because it also verifies that what we are
    //! talking to behaves like a BS210 sign.

    use super::Record;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref UNKNOWN_QUERY_0 : Record = Record {
            data: vec![ 0x06, 0x01, 0x21, 0x00, 0x00, 0x00, 0x00, 0xd8 ]
        };

        static ref UNKNOWN_QUERY_1 : Record = Record {
            data: vec![ 0x04, 0x08, 0x00, 0x20, 0x01, 0xd3 ]
        };

        static ref UNKNOWN_QUERY_2 : Record = Record {
            data: vec![ 0x04, 0x08, 0xd0, 0x1f, 0x10, 0xf5 ]
        };

        static ref PANEL_VERSION : Record = Record {
            data: vec![ 0x04, 0x08, 0xd0, 0x9f, 0x10, 0x75 ]
        };

        static ref UNKNOWN_QUERY_3 : Record = Record {
            data: vec![ 0x01, 0x0f, 0xf0 ]
        };
    }

    /// First record to be sent after selecting the address.
    ///
    /// Device should send back `4f` when sending this query.
    ///
    /// It is not known what the query or the response actually mean.
    pub fn unknown_query_0() -> &'static Record {
        &UNKNOWN_QUERY_0
    }

    /// Second record to be sent after selecting the address.
    ///
    /// Device should send back `4f 01 57 a8` when sending this query.
    ///
    /// It is not known what the query or the response actually mean.
    pub fn unknown_query_1() -> &'static Record {
        &UNKNOWN_QUERY_1
    }

    /// Third record to be sent after selecting the address.
    ///
    /// Device should send back
    /// `4f 10 00 00 02 00 df ff ff ff 00 00 00 00 ff ff f7 f7 26`
    /// when sending this query.
    ///
    /// It is not known what the query or the response actually mean, but
    /// since the payload differs only in one byte from the panel version
    /// query, it would seem that this also tells us something about the
    /// device.
    pub fn unknown_query_2() -> &'static Record {
        &UNKNOWN_QUERY_2
    }

    /// Seems to request some kind of version from the flipdot display,
    /// because the observed payload of the response is `b"PANEL V3.11     "`.
    ///
    /// As always, it is unclear what it means to have panel version 3.11.
    pub fn panel_version() -> &'static Record {
        &PANEL_VERSION
    }

    /// Fifth record to be sent after selecting the address, that is, after
    /// querying the panel version.
    ///
    /// The response seems to be a simple `4f` acknowledgement.
    ///
    /// It is not known what the query or the response actually mean.
    pub fn unknown_query_3() -> &'static Record {
        &UNKNOWN_QUERY_3
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

        #[test]
        fn unknown_query_2_integrity() {
            let static_unkown_query = unknown_query_2();
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
    use super::{Record, checksum, Result, Error};

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
            data.push(checksum(&data));

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

/// A buffer containing one or more messages in the protocol used to flash BS210 signs.
///
/// One kind of message is a chunk of a record database that corresponds to a single
/// line in a sign database of a BS210 sign, but in a different format that is understood
/// by the sign when sending the data over the wire, rather then when viewing it in an editor.
///
/// Also used for clearing the device and for querying some version information.
///
/// There are also kinds of messages with an unclear meaning.
pub struct Record {
    /// Buffer containing the messages. Guaranteed to be sized 2 bytes or longer.
    data: Vec<u8>,
}

impl Record {
    /// The bytes of the full record, including the lengths and the checksums of all contained messages.
    ///
    /// Guaranteed to have a size of two bytes or more.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..]
    }

    /// Record data excluding the first (length) and last (checksum) bytes.
    ///
    /// If there are multiple messages in the record, returns only the first
    /// payload.
    ///
    /// Only use this method for tests.
    #[cfg(test)]
    pub fn payload(&self) -> &[u8] {
        let record_len = self.data[0] as usize;
        assert!(
            self.data.len() >= 2 + record_len,
            "Expected space for the full record, including payload, length and checksum byte"
        );
        &self.data[1..1 + record_len]
    }

    /// Gets the checksum from the message, at the expected position.
    ///
    /// If there are multiple messages in the record, returns only the first
    /// payload.
    ///
    /// Only use this method for tests.
    #[cfg(test)]
    pub fn checksum(&self) -> u8 {
        let record_len = self.data[0] as usize;
        assert!(
            self.data.len() >= 2 + record_len,
            "Expected space for the full record, including payload, length and checksum byte"
        );
        self.data[1 + record_len]
    }
}

mod builder;
mod checksum;
mod error;

use builder::Builder;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub use db::DatabaseChunk;

pub mod db;
pub mod query;
pub mod res;

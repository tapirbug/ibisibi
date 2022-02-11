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
    /// The bytes of the full record, including the length and the checksum.
    ///
    /// Guaranteed to have a size of two bytes or more.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..]
    }

    /// Record data excluding the first (length) and last (checksum) bytes.
    #[cfg(test)]
    pub fn payload(&self) -> &[u8] {
        &self.data[1..self.data.len() - 1]
    }

    #[cfg(test)]
    pub fn checksum(&self) -> u8 {
        self.data[self.data.len() - 1]
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

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Record length out of bounds")]
    RecordLengthOutOfBounds,
    #[error("Response from sign has length that is out of bounds: {len}")]
    ResponseRecordLengthOutOfBounds { len: usize },
    #[error("Response from sign corrupt, lacking magic number")]
    ResponseMagicNumberMissing,
    /// Expected a response holding just the magic number, but got a complex response.
    #[error("Acknowledgement expected but got complex response from sign")]
    ResponseNotAcknowledgement,
    #[error("Response from sign is too short, missing header, trailer, or both")]
    ResponseHeaderOrTrailerMissing,
    #[error(
        "Response from sign corrupt, expected record length: {expected:X?}, got: {received:X?}"
    )]
    ResponsePayloadLenMismatch { expected: u8, received: u8 },
    #[error("Response from sign corrupt, expected checksum: {expected:X?}, got: {received:X?}")]
    ResponseChecksumMismatch { expected: u8, received: u8 },
}

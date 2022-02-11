//! Query messages that are sent to obtain information from the sign.
//!
//! It is not known if it is necessary to make these queries to start the flashing
//! process, but we do it in any case because it also verifies that what we are
//! talking to behaves like a BS210 sign.

use super::Record;
use lazy_static::lazy_static;

lazy_static! {
    static ref PREPARE_CLEAR_0: Record = Record {
        data: vec![0x06, 0x01, 0x21, 0x00, 0x00, 0x00, 0x00, 0xd8]
    };
    static ref PREPARE_CLEAR_1: Record = Record {
        data: vec![0x04, 0x08, 0x00, 0x20, 0x01, 0xd3]
    };
    static ref CLEAR: Record = Record {
        data: vec![
            0x23, 0x03, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xba,
        ]
    };
    static ref FINISH_CLEAR_0: Record = Record {
        data: vec![0x05, 0x05, 0x00, 0x00, 0x00, 0x00, 0xf6]
    };
    static ref FINISH_CLEAR_1: Record = Record {
        data: vec![0x02, 0x07, 0x00, 0xf7]
    };
    static ref FINISH_FLASH_0: Record = Record {
        data: vec![0x02, 0x15, 0x55, 0x94]
    };
    static ref FINISH_FLASH_1: Record = Record {
        data: vec![0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0]
    };
}

/// First record to be sent after selecting the address.
///
/// Device should send back `4f` when sending this query.
///
/// It is not known what the query or the response actually mean, but it was sent in all
/// observed runs of the flashing.
pub fn prepare_clear_0() -> &'static Record {
    &PREPARE_CLEAR_0
}

/// Second record to be sent after selecting the address.
///
/// Device should send back `4f 01 57 a8` when sending this query.
///
/// It is not known what the query or the response actually mean, but it was sent in all
/// observed runs of the flashing.
///
/// After this we see in the logs that we disconnect and connect again, maybe to change
/// baudrate. Not known if this reconnecting is necessary.
pub fn prepare_clear_1() -> &'static Record {
    &PREPARE_CLEAR_1
}

/// Sent four times after `prepare_clear_1`.
///
/// Each time we expect a repsonse of 0x45 (E).
///
/// Why the exact same message is sent four times is not known. Maybe this is supposed
/// to overwrite four consecutive blocks but an implementation error causes it to clear
/// the same block over and over?
pub fn clear() -> &'static Record {
    &CLEAR
}

/// First record to be sent after sending the four clear messages.
///
/// Device should send back `0x4f` (O) when sending this query.
///
/// It is not known what the query or the response actually mean, but it was sent in all
/// observed runs of the flashing.
pub fn finish_clear_0() -> &'static Record {
    &FINISH_CLEAR_0
}

/// Second record to be sent after sending the four clear messages.
///
/// Device should send back `0x4f` (O) when sending this query.
///
/// It is not known what the query or the response actually mean, but it was sent in all
/// observed runs of the flashing.
pub fn finish_clear_1() -> &'static Record {
    &FINISH_CLEAR_1
}

/// First record to be sent after sending the sign database.
///
/// Device should send back `0x4f` (O) when sending this query.
///
/// It is not known what the query or the response actually mean, but it was sent in all
/// observed runs of the flashing.
pub fn finish_flash_0() -> &'static Record {
    &FINISH_FLASH_0
}

/// Second (and last) record to be sent after sending the sign database. Looks like the
/// record is sent four times in sequence, unsuccessfully trying to read a repsponse from
/// the device (timoeout).
///
/// Not clear if this is in any way required for flashing.
pub fn finish_flash_1() -> &'static Record {
    &FINISH_FLASH_1
}

#[cfg(test)]
mod test {
    use super::super::Builder;
    use super::*;

    /// Since length and checksum are handcoded, we need to make sure that
    /// we wrote everything down correctly by calculating a checksum over
    /// the buffers.
    #[test]
    fn prebuilt_queries_integrity() {
        fn check_integrity(query: &'static str, record: &Record) {
            assert!(
                record.data.len() >= 2,
                "Static guarantee for at least two bytes violated in prebuilt query {}",
                query
            );
            let prebuilt_checksum = record.checksum();
            let calculated_checksum_from_content = Builder::new()
                .buf(record.payload())
                .build()
                .unwrap()
                .checksum();
            assert_eq!(
                prebuilt_checksum, calculated_checksum_from_content,
                "Unexpected checksum for query {}",
                query
            )
        }

        check_integrity("prepare_clear_0", prepare_clear_0());
        check_integrity("prepare_clear_1", prepare_clear_1());
        check_integrity("clear", clear());
        check_integrity("finish_clear_0", finish_clear_0());
        check_integrity("finish_clear_1", finish_clear_1());
        check_integrity("finish_flash_0", finish_flash_0());
        check_integrity("finish_flash_1", finish_flash_1());
    }
}

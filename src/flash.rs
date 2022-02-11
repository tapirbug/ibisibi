use crate::{
    args::Flash,
    record::{db::DatabaseChunk, query, res},
    serial::{self, Serial},
    status::status,
    telegram::Telegram,
};
use ihex::{Reader, Record};
use std::{
    fs::read_to_string,
    io::{Read, Write},
};
use thiserror::Error;
use tracing::{debug, warn};

pub type Result<T> = std::result::Result<T, FlashError>;

#[tracing::instrument]
pub fn flash(opts: Flash) -> Result<()> {
    let Flash {
        address,
        sign_db_hex,
        serial,
    } = opts;

    let mut serial = serial::open(&serial).map_err(|e| FlashError::Serial {
        source: e,
        port: serial.clone(),
    })?;
    let db = read_to_string(sign_db_hex).map_err(FlashError::DbRead)?;
    let db = Reader::new(&db);

    check_compatibility(&mut serial, address)?;
    perform_flashing(&mut serial, address, db)
}

/// Ensure that a device is listening at the specified address for flashing, so
/// that we can abort early on obvious operator or connection errors.
///
/// More sanity checks may be added to this function in the future.
#[tracing::instrument(skip(serial))]
fn check_compatibility(serial: &mut Serial, address: u8) -> Result<()> {
    // Check device status first and print it as debug output,
    dump_status(serial, address)

    // Other commands are sent in observed flashings that might
    // also serve as sanity checks, but we do not understand them well
    // enoug to add them here yet.
}

#[tracing::instrument(skip(serial))]
fn dump_status(serial: &mut Serial, address: u8) -> Result<()> {
    let status = status(serial, address)?;
    debug!("Device status before starting flashing: {}", status);
    Ok(())
}

/// Sends the actual flashing commands over the wire.
#[tracing::instrument(skip(serial, db))]
fn perform_flashing(serial: &mut Serial, address: u8, db: Reader) -> Result<()> {
    select_address(serial, address)?;
    clear_database(serial)?;
    flash_database(serial, db)
}

#[tracing::instrument(skip(serial))]
fn select_address(serial: &mut Serial, address: u8) -> Result<()> {
    serial.write_all(Telegram::empty().as_bytes())?;
    // r.S1 (select address?)
    serial.write_all(Telegram::bs_select_address(address).as_bytes())?;
    // no response expected
    Ok(())
}

#[tracing::instrument(skip(serial))]
fn clear_database(serial: &mut Serial) -> Result<()> {
    let mut buf = [0_u8; 4];

    debug!("Preparing clearing (1/2)");
    serial.write_all(query::prepare_clear_0().as_bytes())?;
    serial.read_exact(&mut buf[0..1])?;
    res::verify_ack_response(&buf[0..1]).map_err(FlashError::PrepareClear0)?;

    debug!("Preparing clearing (2/2)");
    const EXPECTED_QUERY_1_RESPONSE: &[u8] = &[0x57];
    serial.write_all(query::prepare_clear_1().as_bytes())?;
    serial.read_exact(&mut buf[..])?;
    let unknown_query_1_response =
        res::response_payload(&buf[..]).map_err(FlashError::PrepareClear1CorruptResponse)?;
    if unknown_query_1_response != EXPECTED_QUERY_1_RESPONSE {
        return Err(FlashError::PrepareClear1);
    }

    for i in 0..4 {
        debug!("Clearing ({}/4)", i);
        serial.write_all(query::clear().as_bytes())?;
        serial.read_exact(&mut buf[0..1])?;
        let response = buf[0];
        if response != b'E' {
            return Err(FlashError::Clear(response));
        }
    }

    debug!("Finishing clearing (1/2)");
    serial.write_all(query::finish_clear_0().as_bytes())?;
    serial.read_exact(&mut buf[0..1])?;
    res::verify_ack_response(&buf[0..1]).map_err(FlashError::FinishClear0)?;

    debug!("Finishing clearing (2/2)");
    serial.write_all(query::finish_clear_1().as_bytes())?;
    serial.read_exact(&mut buf[0..1])?;
    res::verify_ack_response(&buf[0..1]).map_err(FlashError::FinishClear1)?;

    Ok(())
}

#[tracing::instrument(skip(serial, reader))]
fn flash_database(serial: &mut Serial, reader: Reader) -> Result<()> {
    let mut buf = [0_u8; 1];
    let mut eof_found = false;
    let mut write_offset = 0;
    for record in reader {
        let record = record?;
        if eof_found {
            return Err(FlashError::DbUnexpectedRecordType);
        }
        match record {
            Record::Data { value: data, .. } => {
                debug!(
                    "Flashing {len} bytes at offset 0x{offset:X?}",
                    len = data.len(),
                    offset = write_offset
                );

                serial.write_all(
                    DatabaseChunk::new(write_offset, &data)
                        .map_err(FlashError::DbRecordTooLong)?
                        .as_bytes(),
                )?;

                serial.read_exact(&mut buf)?;
                res::verify_ack_response(&buf).map_err(FlashError::FlashChunkNotAcknowledged)?;

                write_offset += 0x20;
            }
            Record::EndOfFile => {
                eof_found = true;
            }
            _ => return Err(FlashError::DbUnexpectedRecordType),
        }
    }

    if !eof_found {
        warn!("No EOF record found in database, ignoring");
    }

    debug!("Finishing flashing (1/2)");
    serial.write_all(query::finish_flash_0().as_bytes())?;
    serial.read_exact(&mut buf)?;
    res::verify_ack_response(&buf).map_err(FlashError::FinishFlash0)?;

    debug!("Finishing flashing (2/2)");
    serial.write_all(query::finish_flash_1().as_bytes())?;
    // do not expect any reponse for the second finishing step

    Ok(())
}

#[derive(Debug, Error)]
pub enum FlashError {
    #[error("Failed to read sign database, error: {0}")]
    DbRead(std::io::Error),
    #[error("Failed to read sign database, error: {0}")]
    DbCorrupt(#[from] ihex::ReaderError),
    #[error("Failed to read sign database, error: {0}")]
    DbRecordTooLong(crate::record::Error),
    #[error(
        "Failed to read sign database, error: unrecognized format, found unexpected record type"
    )]
    DbUnexpectedRecordType,
    #[error("Database record sent, but device failed to send acknowledgement: {0}")]
    FlashChunkNotAcknowledged(crate::record::Error),
    #[error(
        "Flashing could not be finished, unexpected repsonse from device at finsihing step 0: {0}"
    )]
    FinishFlash0(crate::record::Error),
    #[error("Could not open serial port connection to: {port}, due to error: {source}")]
    Serial {
        source: serialport::Error,
        port: String,
    },
    #[error("Failed to write to serial port, error: {0}")]
    SerialWrite(#[from] std::io::Error),
    #[error("{0}")]
    IbisResponseCorrupt(#[from] crate::telegram::TelegramParseError),
    #[error("Could not check device status before clearing and flashing, error: {0}")]
    Status(#[from] crate::status::Error),
    #[error("Could not clear sign database, unexpected response from device at clearing preparation step 0")]
    PrepareClear0(crate::record::Error),
    #[error("Could not clear sign database, unexpected response from device at clearing preparation step 1, error: {0}")]
    PrepareClear1CorruptResponse(crate::record::Error),
    #[error("Could not clear sign database, unexpected response from device at clearing preparation step 1")]
    PrepareClear1,
    #[error("Could not clear sign database, unexpected response: {0}")]
    Clear(u8),
    #[error("Could not clear sign database, unexpected response from device at clearing finishing step 0, error: {0}")]
    FinishClear0(crate::record::Error),
    #[error("Could not clear sign database, unexpected response from device at clearing finishing step 1, error: {0}")]
    FinishClear1(crate::record::Error),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::serial::Serial;

    #[test]
    fn check_compatibility_timeout() {
        let mut serial = Serial::builder().expect_write(b"a1\r\"").time_out().build();

        match check_compatibility(&mut serial, 1) {
            Err(FlashError::Status(_)) => {}
            other => panic!(
                "Expected status error, but got Ok or unexpected variant: {:?}",
                other
            ),
        }
    }

    #[test]
    fn check_compatibility_checksum_err() {
        let mut serial = Serial::builder()
            .expect_write(b"a1\r\"")
            .respond(b"a3\r?") // correct checksum would be a space (0x20)
            .build();

        match check_compatibility(&mut serial, 1) {
            Err(FlashError::Status(_)) => {}
            other => panic!(
                "Expected status error, but got Ok or unexpected variant: {:?}",
                other
            ),
        }
    }

    #[test]
    fn check_compatibility_ok() {
        let mut serial = Serial::builder()
            .expect_write(b"a1\r\"")
            .respond(b"a3\r ")
            .build();

        match check_compatibility(&mut serial, 1) {
            Ok(()) => {}
            Err(err) => panic!(
                "Expected status query to be Ok but got unexpected error: {:?}",
                err
            ),
        }
    }

    /// Tests that an attempt to flash mini0 reproduces what we observed during actual flashing.
    #[test]
    fn flash_mini0_happy_path() {
        const MINI0: &str =
            ":20000000570012001B00121C8B4506F900E001000AE001050A0080016001A0004F00003083
:200020000D0D0D0D0D0D0D0D0D0D0D0D0D0D0D00000000E001000A004F004F004F004F00D6
:100040004F00004F0000000000000000000000FF13
:12006000464E543A20674255534530202D20312E323157
:20008000E0000841030470A070FF00FF0000000000000000000000000000000000000000B2
:2000A0000000000000000000000000000041000000000000000000000000000000000000FF
:2000C000000000000000000000000000000000000000000000000000000000000000000020
:2000E000000000000000000000000000000000000000000000000000000000000000000000
:200100000000000000000000000000000000000000000000000000000000000000000000DF
:0D012000000000000000000000000000FFD3
:120140004C494E3A20674255534530202D20312E32317B
:0E0160003030310800E0B0C01B7310410DFFBD
:1201800043494C3A20674255534530202D20312E323146
:0D01A0003030310700E0B0C04141410DFF9B
:00000001FF
";
        let reader = Reader::new(MINI0);
        let mut serial = Serial::builder()
            // The initial address selection, no response expected
            .expect_write(&[0x0d, 0x72])
            .expect_write(&[0x1b, 0x53, 0x31, 0x0d, 0x0b])
            // Clearing setup 1
            .expect_write(&[0x06, 0x01, 0x21, 0x00, 0x00, 0x00, 0x00, 0xd8])
            .respond(b"O")
            // Clearing setup 2
            .expect_write(&[0x04, 0x08, 0x00, 0x20, 0x01, 0xd3])
            .respond(&[0x4f, 0x01, 0x57, 0xa8])
            // Actual clearing (yes, four times the same message)
            .expect_write(&[
                0x23, 0x03, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xba,
            ])
            .respond(b"E")
            .expect_write(&[
                0x23, 0x03, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xba,
            ])
            .respond(b"E")
            .expect_write(&[
                0x23, 0x03, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xba,
            ])
            .respond(b"E")
            .expect_write(&[
                0x23, 0x03, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xba,
            ])
            .respond(b"E")
            // Finish clearing 1
            .expect_write(&[0x05, 0x05, 0x00, 0x00, 0x00, 0x00, 0xf6])
            .respond(b"O")
            // Finish clearing 2
            .expect_write(&[0x02, 0x07, 0x00, 0xf7])
            .respond(b"O")
            // Sign database data
            .expect_write(&[
                0x24, 0x05, 0x00, 0x00, 0x00, 0x57, 0x00, 0x12, 0x00, 0x1b, 0x00, 0x12, 0x1c, 0x8b,
                0x45, 0x06, 0xf9, 0x00, 0xe0, 0x01, 0x00, 0x0a, 0xe0, 0x01, 0x05, 0x0a, 0x00, 0x80,
                0x01, 0x60, 0x01, 0xa0, 0x00, 0x4f, 0x00, 0x00, 0x30, 0x7a,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0x20, 0x00, 0x00, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d,
                0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x0d, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x01, 0x00, 0x0a,
                0x00, 0x4f, 0x00, 0x4f, 0x00, 0x4f, 0x00, 0x4f, 0x00, 0xcd,
            ])
            .respond(b"O")
            .expect_write(&[
                0x14, 0x05, 0x40, 0x00, 0x00, 0x4f, 0x00, 0x00, 0x4f, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x0a,
            ])
            .respond(b"O")
            .expect_write(&[
                0x16, 0x05, 0x60, 0x00, 0x00, 0x46, 0x4e, 0x54, 0x3a, 0x20, 0x67, 0x42, 0x55, 0x53,
                0x45, 0x30, 0x20, 0x2d, 0x20, 0x31, 0x2e, 0x32, 0x31, 0x4e,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0x80, 0x00, 0x00, 0xe0, 0x00, 0x08, 0x41, 0x03, 0x04, 0x70, 0xa0, 0x70,
                0xff, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa9,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0xa0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x17,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf7,
            ])
            .respond(b"O")
            .expect_write(&[
                0x24, 0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd6,
            ])
            .respond(b"O")
            .expect_write(&[
                0x11, 0x05, 0x20, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xff, 0xca,
            ])
            .respond(b"O")
            .expect_write(&[
                0x16, 0x05, 0x40, 0x01, 0x00, 0x4c, 0x49, 0x4e, 0x3a, 0x20, 0x67, 0x42, 0x55, 0x53,
                0x45, 0x30, 0x20, 0x2d, 0x20, 0x31, 0x2e, 0x32, 0x31, 0x72,
            ])
            .respond(b"O")
            .expect_write(&[
                0x12, 0x05, 0x60, 0x01, 0x00, 0x30, 0x30, 0x31, 0x08, 0x00, 0xe0, 0xb0, 0xc0, 0x1b,
                0x73, 0x10, 0x41, 0x0d, 0xff, 0xb4,
            ])
            .respond(b"O")
            .expect_write(&[
                0x16, 0x05, 0x80, 0x01, 0x00, 0x43, 0x49, 0x4c, 0x3a, 0x20, 0x67, 0x42, 0x55, 0x53,
                0x45, 0x30, 0x20, 0x2d, 0x20, 0x31, 0x2e, 0x32, 0x31, 0x3d,
            ])
            .respond(b"O")
            .expect_write(&[
                0x11, 0x05, 0xa0, 0x01, 0x00, 0x30, 0x30, 0x31, 0x07, 0x00, 0xe0, 0xb0, 0xc0, 0x41,
                0x41, 0x41, 0x0d, 0xff, 0x92,
            ])
            .respond(b"O")
            .expect_write(&[0x02, 0x15, 0x55, 0x94])
            .respond(b"O")
            // no response expected for this, not sure if relevant
            .expect_write(&[
                0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0, 0x01, 0x0f, 0xf0,
            ])
            .build();

        perform_flashing(&mut serial, 1, reader).expect("flashing should succeed here");
    }
}

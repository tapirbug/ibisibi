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

    for _ in 0..4 {
        debug!("Finishing flashing (2/2)");
        serial.write_all(query::finish_flash_1().as_bytes())?;
    }
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

use crate::{
    args::Flash,
    serial::{self, Serial},
    telegram::Telegram,
    record::{db, query, res},
    status::{status, Status}
};
use thiserror::Error;
use std::{
    io::{Read, Write},
    fs::read_to_string,
    convert::TryFrom
};
use ihex::Reader;
use tracing::debug;

pub type Result<T> = std::result::Result<T, FlashError>;

#[tracing::instrument]
pub fn flash(opts: Flash) -> Result<()> {
    let Flash { address, sign_db_hex, serial } = opts;

    let mut serial = serial::open(&serial)
        .map_err(|e| FlashError::Serial { source: e, port: serial.clone() })?;

    select_address(&mut serial, address)?;
    verify_display_version(&mut serial, address)?;
    check_status(&mut serial, address)?;
    for i in 0..4 {
        clear_database(&mut serial)?;
    }
    flash_database(
        &mut serial,
        Reader::new(&read_to_string(sign_db_hex)?)
    )?;

    todo!("missing two commands after flashing")
}

#[tracing::instrument(skip(serial))]
fn select_address(serial: &mut Serial, address: u8) -> Result<()> {
    // r.S1 (select address?)
    serial.write(
        Telegram::bs_select_address(address).as_bytes()
    )?;
    // no response expected
    Ok(())
}

/// Sends several queries to the flipdot display and verifies that the responses are
/// consistent with what we would expect from a BS210 flipdot display.
///
/// Not all of the queries are well-understood, but we can check that the responses
/// are consistent with what we have observed from a BS210 sign in the past.
#[tracing::instrument(skip(serial))]
fn verify_display_version(serial: &mut Serial, address: u8) -> Result<()> {
    let mut buf = [0; 32];

    debug!("Unknown query 0 start, expecting acknowledgement back");
    serial.write(
        query::unknown_query_0().as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    res::verify_ack_response(raw_response)?;

    debug!("Unknown query 1 start, expecting response 0x57");
    const EXPECTED_QUERY_1_RESPONSE: &[u8] = &[ 0x57 ];
    serial.write(
        query::unknown_query_1().as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    let unknown_query_1_response = res::response_payload(raw_response)?;
    if unknown_query_1_response != EXPECTED_QUERY_1_RESPONSE {
        return Err(FlashError::unknown_query_1(raw_response));
    }

    debug!("Unknown query 2 start, expecting response 00 00 02 00 df ff ff ff 00 00 00 00 ff ff f7 f7");
    const EXPECTED_QUERY_2_RESPONSE: &[u8] = &[ 0x00, 0x00, 0x02, 0x00, 0xdf, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xf7, 0xf7 ];
    serial.write(
        query::unknown_query_2().as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    let unknown_query_2_response = res::response_payload(raw_response)?;
    if unknown_query_2_response != EXPECTED_QUERY_2_RESPONSE {
        return Err(FlashError::unknown_query_2(raw_response));
    }

    debug!("Version query, expecting response \"PANEL V3.11     \"");
    const EXPECTED_PANEL_VERSION_RESPONSE: &[u8] = b"PANEL V3.11     ";
    serial.write(
        query::panel_version().as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    let panel_version_response = res::response_payload(raw_response)?;
    if panel_version_response != EXPECTED_PANEL_VERSION_RESPONSE {
        return Err(FlashError::panel_version_ibis(raw_response));
    }

    debug!("Unknown query 3 start, expecting acknowledgement back");
    serial.write(
        query::unknown_query_3().as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    res::verify_ack_response(raw_response)?;

    debug!("IBIS display version query start, expecting response \"aVV2.3RigaB/H7/99\"");
    serial.write(
        Telegram::display_version(address).as_bytes()
    )?;
    let bytes_read = serial.read(&mut buf)?;
    let raw_response = &buf[0..bytes_read];
    if Telegram::try_from(raw_response)?.payload() != b"aVV2.3RigaB/H7/99" {
        return Err(FlashError::panel_version(raw_response));
    }
    todo!("Check for response \"aVV2.3RigaB/H7/99\" but also check checksume tc")
}

#[tracing::instrument(skip(serial))]
pub fn check_status(serial: &mut Serial, address: u8) -> Result<()> {
    match status(serial, address)? {
        Status::ReadyForData => Ok(()),
        other => Err(FlashError::NotReadyForData(other))
    }
}

#[tracing::instrument(skip(serial))]
pub fn clear_database(serial: &mut Serial) -> Result<()> {
    todo!()
}

#[tracing::instrument(skip(serial, reader))]
pub fn flash_database(serial: &mut Serial, reader: Reader) -> Result<()> {
    todo!()
}

#[tracing::instrument(skip(serial, database))]
fn flash_db_bytes(serial: &mut Serial, address: u8, database: &[u8]) {
    // 1. query status to find right address
    // r.S1 (select address? no response expected)
    // write weird record 1: 06 01 21 00 00 00 00 d8
    // expect response: 4f
    // write weird record 2: 04 08 00 20 01 d3
    // expect response: 4f 01 57 a8

    // OPTIONAL PART START

    // write weird record 3: 04 08 d0 1f 10 f5
    // expect response: 4f 10 00 00 02 00 df ff ff ff 00 00 00 00 ff ff f7 f7 26
    // write record to query some kind of version: 04 08 d0 9f 10 75
    // expect response:
    // "O.PANEL V3.11      §" 4f 10 50 41 4e 45 4c 20 56 33 2e 31 31 20 20 20 20 20 a7
    // write wierd record 4: 01 0f f0
    // expect response: 4f

    // query display version with IBIS: 61 56 31 0d 74                                    aV1.t
    // get back: aVV2.3RigaB/H7/99

    // r.S1 again (select address? no response expected)

    // OPTIONAL PART END
}

#[derive(Debug, Error)]
pub enum FlashError {
    #[error("Connected device does not behave like BS210 - Response to {query_desc} was unexpected: {response:X?}")]
    DisplayVersion {
        query_desc: &'static str,
        response: Vec<u8>
    },
    #[error("Failed to read sign database, error: {0}")]
    Db(std::io::Error),
    #[error("Could not open serial port connection to: {port}, due to error: {source}")]
    Serial {
        source: serialport::Error,
        port: String,
    },
    #[error("Failed to write to serial port, error: {0}")]
    SerialWrite(#[from] std::io::Error),
    #[error("{0}")]
    Record(#[from] crate::record::Error),
    #[error("{0}")]
    IbisResponseCorrupt(#[from] crate::telegram::TelegramParseError),
    #[error("Expected device to be ready for data, but got unexpected status {0}")]
    NotReadyForData(Status),
    #[error("{0}")]
    Status(#[from] crate::status::Error),
}

impl FlashError {
    pub fn unknown_query_1(unexpected_response: &[u8]) -> FlashError {
        Self::DisplayVersion {
            query_desc: "unknown query 1",
            response: unexpected_response.into()
        }
    }

    pub fn unknown_query_2(unexpected_response: &[u8]) -> FlashError {
        Self::DisplayVersion {
            query_desc: "unknown query 2",
            response: unexpected_response.into()
        }
    }

    pub fn panel_version(unexpected_response: &[u8]) -> FlashError {
        Self::DisplayVersion {
            query_desc: "panel_version",
            response: unexpected_response.into()
        }
    }

    pub fn panel_version_ibis(unexpected_response: &[u8]) -> FlashError {
        Self::DisplayVersion {
            query_desc: "panel version (IBIS)",
            response: unexpected_response.into()
        }
    }
}
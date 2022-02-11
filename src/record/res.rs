use super::{checksum::checksum, Error, Result};

/// Verifies that the given buffer holds an acknowledgement response without an attached
/// record, that is 0x4F.
pub fn verify_ack_response(buf: &[u8]) -> Result<()> {
    if buf.is_empty() || buf[0] != 0x4f {
        return Err(Error::ResponseMagicNumberMissing);
    }

    if buf != [0x4f] {
        return Err(Error::ResponseNotAcknowledgement);
    }

    Ok(())
}

/// Verifies that a reponse from a BS210 conforms to the normal structure of a response
/// received from BS210, that is, it starts with 0x4f, followed by a record. Returns only
/// the payload part of the response if successfull, that is, 0x4f, len and checksum are left
/// out.
pub fn response_payload(buf: &[u8]) -> Result<&[u8]> {
    if buf.is_empty() || buf[0] != 0x4f {
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
            received: received_checksum,
        });
    }

    let buf_payload_len = buf[0];
    let buf = &buf[1..];
    if buf.len() > 0xFF {
        return Err(Error::ResponseRecordLengthOutOfBounds { len: buf.len() });
    }
    let received_payload_len = buf.len() as u8;
    if received_payload_len != buf_payload_len {
        return Err(Error::ResponsePayloadLenMismatch {
            expected: buf_payload_len,
            received: received_payload_len,
        });
    }

    Ok(buf)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ok_unknown_query_0_response() {
        const RESPONSE: &[u8] = &[0x4f, 0x01, 0x57, 0xa8];
        response_payload(RESPONSE).unwrap();
    }

    #[test]
    fn ok_other_response_of_unknown_purpose() {
        const RESPONSE: &[u8] = &[
            0x4f, 0x10, 0x00, 0x00, 0x02, 0x00, 0xdf, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xf7, 0xf7, 0x26,
        ];
        response_payload(RESPONSE).unwrap();
    }

    #[test]
    fn ok_panel_v_3_11_response() {
        const RESPONSE: &[u8] = &[
            0x4f, 0x10, 0x50, 0x41, 0x4e, 0x45, 0x4c, 0x20, 0x56, 0x33, 0x2e, 0x31, 0x31, 0x20,
            0x20, 0x20, 0x20, 0x20, 0xa7,
        ];
        response_payload(RESPONSE).unwrap();
    }

    #[test]
    fn checksum_missing_unknown_query_0_response() {
        const RESPONSE: &[u8] = &[0x4f, 0x01, 0x57];
        assert_eq!(
            response_payload(RESPONSE).unwrap_err(),
            Error::ResponseChecksumMismatch {
                expected: 0xFF,
                received: 0x57
            }
        )
    }

    #[test]
    fn checksum_failure_unknown_query_0_response() {
        const RESPONSE: &[u8] = &[0x4f, 0x01, 0x57, 0xb9];
        assert_eq!(
            response_payload(RESPONSE).unwrap_err(),
            Error::ResponseChecksumMismatch {
                expected: 0xa8,
                received: 0xb9
            }
        )
    }

    #[test]
    fn ok_ack() {
        const RESPONSE: &[u8] = &[0x4f];
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
        const RESPONSE: &[u8] = &[0x5f];
        assert_eq!(
            verify_ack_response(RESPONSE).unwrap_err(),
            Error::ResponseMagicNumberMissing
        )
    }

    #[test]
    fn ack_with_extra_bytes() {
        const RESPONSE: &[u8] = &[0x4f, 0x00];
        assert_eq!(
            verify_ack_response(RESPONSE).unwrap_err(),
            Error::ResponseNotAcknowledgement
        )
    }

    #[test]
    fn empty_response() {
        assert_eq!(
            response_payload(&[]).unwrap_err(),
            Error::ResponseMagicNumberMissing
        )
    }
}

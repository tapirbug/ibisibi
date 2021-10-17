/// A chunk of a record database that corresponds to a single line in a sign database
/// of a BS210 sign, but in a different format that is understood by the sign when
/// sending the data over the wire, rather then when viewing it in an editor.
struct Record {
    data: Vec<u8>
}

type Result<T> = std::result::Result<T, Error>;

impl Record {
    pub fn new(address: u16, content: &[u8]) -> Result<Self> {
        if content.len() > 0xFF {
            return Err(Error::RecordLengthOutOfBounds)
        }
        let mut data = Vec::new();
        // 1 byte record length
        data.push(content.len() as u8 + 4);
        // 1 byte 5 (unknown purpose)
        data.push(0x05);
        // 2 bytes address (little endian)
        let address = address.to_le_bytes();
        data.push(address[0]);
        data.push(address[1]);
        // record type 0 (data record)
        data.push(0);
        // then the content
        data.extend(content);
        // then a checksum
        data.push(calculate_checksum(&data[..]));
        let record = Record { data };
        Ok(record)
    }

    /// Length only of the data part (length byte and checksum byte not included).
    pub fn data_len(&self) -> u8 {
        self.data[0]
    }

    pub fn address(&self) -> u16 {
        u16::from_le_bytes([self.data[2], self.data[3]])
    }

    /// The data part of the record.
    pub fn data(&self) -> &[u8] {
        &self.data[5..(self.data.len() - 1)]
    }

    pub fn checksum(&self) -> u8 {
        self.data[self.data.len() - 1]
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..]
    }
}

fn calculate_checksum(data: &[u8]) -> u8 {
    !data.iter().cloned().fold(0, u8::wrapping_add) + 1
}

#[derive(Debug)]
pub enum Error {
    RecordLengthOutOfBounds
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mini0_firstrecord() {
        // The raw data part of the first record in mini0.hex
        const DATA: &[u8] = &[
            0x57,
            0x00,
            0x12,
            0x00,
            0x1B,
            0x00,
            0x12,
            0x1C,
            0x8B,
            0x45,
            0x06,
            0xF9,
            0x00,
            0xE0,
            0x01,
            0x00,
            0x0A,
            0xE0,
            0x01,
            0x05,
            0x0A,
            0x00,
            0x80,
            0x01,
            0x60,
            0x01,
            0xA0,
            0x00,
            0x4F,
            0x00,
            0x00,
            0x30
        ];

        let record = Record::new(0, DATA).unwrap();

        assert_eq!(record.data_len(), 0x24, "Unexpected length of resulting data part");
        assert_eq!(record.address(), 0x0, "Unexpected address");
        assert_eq!(record.as_bytes()[2], 0x0, "Unexpected address");
        assert_eq!(record.as_bytes()[3], 0x0, "Unexpected address");
        assert_eq!(record.data(), DATA, "Unexpected content data");
        assert_eq!(record.checksum(), 0x7A, "Expected checksum: 7A but got {:X?}", record.checksum())
    }

    #[test]
    fn mini0_first_record_with_two_byte_address() {
        // The raw data part of the first record in mini0.hex
        const DATA: &[u8] = &[0; 32];

        let record = Record::new(0x0100, DATA).unwrap();

        assert_eq!(record.data_len(), 0x24, "Unexpected length of resulting data part");
        assert_eq!(record.address(), 0x0100, "Unexpected address");
        assert_eq!(record.as_bytes()[2], 0x00, "Unexpected address");
        assert_eq!(record.as_bytes()[3], 0x01, "Unexpected address");
        assert_eq!(record.data(), DATA, "Unexpected content data");
        assert_eq!(record.checksum(), 0xD6, "Expected checksum: D6 but got {:X?}", record.checksum())
    }
}
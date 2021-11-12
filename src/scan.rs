use crate::serial::Serial;
use crate::status::{status, Status};

pub type Result<T> = std::result::Result<T, crate::status::Error>;

pub struct Scan<'a> {
    serial: &'a mut Serial,
    next_address: u8
}

const ADDRESS_MIN : u8 = 0;
const ADDRESS_MAX : u8 = 15;

impl<'a> Scan<'a> {
    pub fn new(serial: &'a mut Serial) -> Self {
        Self { serial, next_address: ADDRESS_MIN }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Find {
    address: u8,
    status: Status
}

impl Find {
    pub fn address(&self) -> u8 {
        self.address
    }

    pub fn status(&self) -> Status {
        self.status
    }
}

impl<'a> Iterator for Scan<'a> {
    type Item = Result<Find>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_address > ADDRESS_MAX {
            return None;
        }

        let address = self.next_address;
        let item = status(self.serial, address)
            .map(|s| Find { address: address, status: s });
        self.next_address += 1;
        Some(item)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn discover_address_9() {
        let mut serial = Serial::builder();
        let available_address = 9;
        for address in ADDRESS_MIN..=ADDRESS_MAX {
            if address != available_address {
                serial.time_out();
            } else {
                serial.receive(b"a0\r#");
            }
        }
        let mut serial = serial.build();
        for (idx, result) in Scan::new(&mut serial).enumerate() {
            if (idx as u8) == available_address {
                let find = result.unwrap();
                assert_eq!(find.address(), available_address);
                assert_eq!(find.status(), Status::ReadyForData);
            } else {
                let err = result.unwrap_err();
                assert!(err.is_timed_out());
            }
        }
    }
}
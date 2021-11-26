use serialport::Result;
#[cfg(not(test))]
use serialport::{new, DataBits, Parity, StopBits};
#[cfg(not(test))]
use std::time::Duration;
use std::{borrow::Cow, convert::Into};

#[cfg(not(test))]
const TIMEOUT_SECS: u64 = 3;

/// Shorter type alias for handles to serial ports.
///
/// Currently the same for all platforms but that might change in the future.
#[cfg(not(test))]
pub type Serial = Box<dyn serialport::SerialPort>;

/// Version of serial ports to use for tests where we choose what the device
/// will respond.
#[cfg(test)]
pub type Serial = mock::MockSerial;

#[cfg(not(test))]
pub fn open<'a, D>(device: D) -> Result<Serial>
where
    D: Into<Cow<'a, str>>,
{
    new(device, 1200)
        .data_bits(DataBits::Seven)
        .stop_bits(StopBits::Two)
        .parity(Parity::Even)
        .timeout(Duration::new(TIMEOUT_SECS, 0))
        .open()
}

#[cfg(test)]
pub fn open<'a, D>(_device: D) -> Result<Serial>
where
    D: Into<Cow<'a, str>>,
{
    todo!("mocking of open function for test currently not needed")
}

#[cfg(test)]
mod mock {
    use std::{
        io::{Error, ErrorKind, Read, Result, Write},
        mem::replace,
    };

    pub struct MockSerial {
        /// We expect these buffers to be written in sequence.
        expected_writes: Vec<Vec<u8>>,
        /// Scheduled responses for the next reads.
        read_results: Vec<ReadResult>,
    }

    impl MockSerial {
        pub fn builder() -> Builder {
            Builder {
                expected_writes: vec![],
                read_results: vec![],
            }
        }
    }

    impl Read for MockSerial {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.read_results.is_empty() {
                // no more mock responses were configured, likely an error in the test setup
                panic!(
                    "No more mock interactions were planned, but mock serial has been read again"
                )
            }

            match self.read_results[0] {
                ReadResult::Timeout => {
                    self.read_results.remove(0);
                    Err(Error::from(ErrorKind::TimedOut))
                }
                ReadResult::Data(ref mut first) => {
                    let first_len = first.len();
                    let first = if first_len <= buf.len() {
                        // whole first vector fits into buf
                        // fully remove from queued read results
                        self.read_results.remove(0)
                    } else {
                        // buf does not have enough capacity for the whole first vector,
                        // get first part and enqeue the rest for later
                        let tail = first.split_off(buf.len());
                        replace(&mut self.read_results[0], ReadResult::Data(tail))
                    };

                    // unwrap the removed read result again
                    let first = match first {
                        ReadResult::Data(data) => data,
                        _ => unreachable!(),
                    };

                    // write first vector or part of first vector,
                    // which may or may not fill the read buffer (but do not read more if there is more space)
                    buf[..first.len()].copy_from_slice(&first);

                    // return amount of read bytes
                    Ok(first.len())
                }
            }
        }
    }

    impl Write for MockSerial {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            if self.expected_writes.is_empty() {
                panic!("Expected no more writes but got {:X?}", buf);
            }

            let expected = self.expected_writes.remove(0);
            if &expected != buf {
                panic!(
                    "Expected to receive {expected:X?} but got {actual:X?}",
                    expected = expected,
                    actual = buf
                );
            }

            // do nothing but fool the code under test that all data has been "written"
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            // do nothing but fool the code under test that all data has been "flushed"
            Ok(())
        }
    }

    impl Drop for MockSerial {
        fn drop(&mut self) {
            if !self.expected_writes.is_empty() {
                // This panic causes an abort if drop is called inside a panic.
                // In such cases the program will abort and omit this helpful message.
                // Hence: If a test expects a panic, ensure that no more writes are scheduled.

                // FIXME un-uncomment
                // panic!("Expected more interactions:\n{:X?}", self.expected_writes.iter().enumerate());
            }
        }
    }

    #[derive(Clone)]
    enum ReadResult {
        Data(Vec<u8>),
        Timeout,
    }

    pub struct Builder {
        read_results: Vec<ReadResult>,
        expected_writes: Vec<Vec<u8>>,
    }

    impl Builder {
        /// Plans that the next write attempt will write exactly the given data.
        pub fn expect_write(&mut self, request: &[u8]) -> &mut Self {
            self.expected_writes.push(request.to_vec());
            self
        }

        /// Plans that the next read attempt will read exactly the given data.
        ///
        /// If it does not fill the buffer completely, the rest will be read later.
        pub fn respond(&mut self, response: &[u8]) -> &mut Self {
            self.read_results.push(ReadResult::Data(response.to_vec()));
            self
        }

        /// Plans the next read attempt to time out.
        pub fn time_out(&mut self) -> &mut Self {
            self.read_results.push(ReadResult::Timeout);
            self
        }

        /// Finish the build.
        ///
        /// Can safely be called multiple times.
        pub fn build(&self) -> MockSerial {
            MockSerial {
                expected_writes: self.expected_writes.clone(),
                read_results: self.read_results.clone(),
            }
        }
    }
}

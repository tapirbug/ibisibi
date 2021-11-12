/// Shorter type alias for handles to serial ports.
///
/// Currently the same for all platforms but that might change in the future.
#[cfg(not(test))]
pub type Serial = Box<serialport::SerialPort>;
#[cfg(test)]
pub type Serial = mock::MockSerial;

#[cfg(test)]
mod mock {
    use std::{
        io::{Read, Write, Error, ErrorKind, Result},
        mem::replace
    };

    pub struct MockSerial {
        read_results: Vec<ReadResult>
    }

    impl Read for MockSerial {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.read_results.is_empty() {
                // no more mock responses were configured, likely an error in the test setup
                panic!("No more mock interactions were planned, but mock serial has been read again")
            }

            match self.read_results[0] {
                ReadResult::Timeout => {
                    self.read_results.remove(0);
                    Err(Error::from(ErrorKind::TimedOut))
                },
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
                        _ => unreachable!()
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
            // do nothing but fool the code under test that all data has been "written"
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            // do nothing but fool the code under test that all data has been "flushed"
            Ok(())
        }
    }

    #[derive(Clone)]
    enum ReadResult {
        Data(Vec<u8>),
        Timeout
    }

    impl MockSerial {
        pub fn builder() -> Builder {
            Builder {
                read_results: vec![]
            }
        }
    }

    pub struct Builder {
        read_results: Vec<ReadResult>
    }

    impl Builder {
        /// Plans that the next read attempt will read exactly the given data.
        ///
        /// If it does not fill the buffer completely, the rest will be read later.
        pub fn receive(&mut self, response: &[u8]) -> &mut Self {
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
                read_results: self.read_results.clone()
            }
        }
    }
}
/// Shorter type alias for handles to serial ports.
/// 
/// Currently the same for all platforms but that might change in the future.
pub type Serial = Box<serialport::SerialPort>;

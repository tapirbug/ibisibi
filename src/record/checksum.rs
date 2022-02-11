pub fn checksum(data: &[u8]) -> u8 {
    (!data.iter().cloned().fold(0, u8::wrapping_add)).wrapping_add(1)
}

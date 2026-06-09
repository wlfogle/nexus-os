//! CRC32 (ISO 3309 / ITU-T V.42) used by GPT header and partition entry checksums.
//! Reflected polynomial 0xEDB88320.

const POLY: u32 = 0xEDB8_8320;

/// Compute CRC32 of `data`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ POLY;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

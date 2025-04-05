//! A module that holds functions related to manipulating data in accordance with BLE specs.

/// Reverse the bit order for each byte in the given `buf`.
///
/// This does not alter the buffer's Endianness.
pub fn reverse_bits(buf: &mut [u8]) {
    for byte in buf {
        *byte = byte.reverse_bits();
    }
}

/// Whiten or de-whiten the given `buf` using the given `coefficient`.
///
/// This is used to avoid transmitting long consecutive repetitions of `0`s and `1`s.
/// The given `coefficient` shall be the index of
/// [`BLE_CHANNEL`](value@crate::radio::BLE_CHANNEL) for the radio channel that
/// transmits or receives the data.
pub fn whiten(buf: &mut [u8], coefficient: u8) {
    let mut coefficient = coefficient;
    for byte in buf {
        let mut result = *byte;
        let mut mask = 1u8;
        for _ in 0u8..8 {
            if (coefficient & 1) == 1 {
                coefficient ^= 0x88;
                result ^= mask;
            }
            mask <<= 1;
            coefficient >>= 1;
        }
        *byte = result;
    }
}

/// Calculate a 24 bit CRC checksum for the given `buf`.
///
/// The returned buffer shall be appended to the transmitted payload
/// *before* applying [`reverse_bits()`] and [`whiten()`].
pub fn crc24_ble(data: &[u8]) -> [u8; 3] {
    let degree_polynomial: u32 = 0x65B;
    let mut crc: u32 = 0x555555;
    for byte in data {
        let copy = *byte;
        crc ^= (copy.reverse_bits() as u32) << 16;
        for _ in 0u8..8 {
            if (crc & 0x800000) > 0 {
                crc = (crc << 1) ^ degree_polynomial;
            } else {
                crc <<= 1;
            }
        }
        crc &= 0xFFFFFF;
    }
    let mut checksum = crc.to_be_bytes();
    reverse_bits(&mut checksum);
    let mut result = [0u8; 3];
    result.copy_from_slice(&checksum[1..4]);
    result
}

#[cfg(test)]
mod test {
    use super::{crc24_ble, reverse_bits, whiten};

    #[test]
    fn reverse() {
        let mut buf = [0u8; 11];
        buf.copy_from_slice(b"Hello World");
        reverse_bits(&mut buf);
        let mut expected = [
            0x12, 0xA6, 0x36, 0x36, 0xF6, 0x04, 0xEA, 0xF6, 0x4E, 0x36, 0x26,
        ];
        assert_eq!(buf, expected);

        reverse_bits(&mut buf);
        expected.copy_from_slice(b"Hello World");
        assert_eq!(buf, expected);
    }

    #[test]
    fn whitening() {
        let coefficient = (2 + 37) | 0x40;

        let mut buf = [0u8; 11];
        buf.copy_from_slice(b"Hello World");
        whiten(&mut buf, coefficient);

        let expected: [u8; 11] = [
            0x57, 0x52, 0x26, 0x33, 0xEA, 0xD6, 0xCB, 0xF5, 0xB3, 0xBA, 0xA1,
        ];
        assert_eq!(buf, expected);

        // de-whiten (w/ same coefficient) should
        // restore the buffer to original content
        whiten(&mut buf, coefficient);
        let mut expected = [0u8; 11];
        expected.copy_from_slice(b"Hello World");
        assert_eq!(buf, expected);
    }

    #[test]
    fn crc() {
        let buffer = b"Hello World";
        let checksum = crc24_ble(buffer);

        // ensure original buffer is unaltered
        assert_eq!(buffer, b"Hello World");

        let expected = [0xB6u8, 0x8C, 0xB0];
        assert_eq!(expected, checksum);
    }
}

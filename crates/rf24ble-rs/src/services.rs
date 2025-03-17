//! A module to contain all compatible BLE services.

#[cfg(feature = "std")]
extern crate std;

/// The Temperature Service UUID number
const TEMPERATURE_UUID: u16 = 0x1809;
/// The Battery Service UUID number
const BATTERY_UUID: u16 = 0x180F;
/// The Eddystone Service UUID number
const EDDYSTONE_UUID: u16 = 0xFEAA;

/// A data service that broadcasts a temperature (in Celsius)
///
/// Conforms to the Health Thermometer Measurement format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[derive(Debug, Clone, Copy)]
pub struct TemperatureService {
    buf: [u8; 6],
}

impl Default for TemperatureService {
    fn default() -> Self {
        Self::new()
    }
}

impl TemperatureService {
    /// Construct an instance of [`TemperatureService`]
    pub fn new() -> Self {
        let mut data = [0u8; 6];
        data[0..2].copy_from_slice(&TEMPERATURE_UUID.to_le_bytes());
        data[5] = 0xFE;
        Self { buf: data }
    }

    /// Set the temperature measurement (in Celsius) data.
    pub fn set_data(&mut self, value: f32) {
        let buf = ((value * 100.0) as u32 & 0xFFFFFF).to_le_bytes();
        self.buf[2..5].copy_from_slice(&buf[0..3]);
    }

    /// Get the temperature measurement (in Celsius) data.
    pub fn data(&self) -> f32 {
        let mut buf = [0u8; 4];
        buf[0..3].copy_from_slice(&self.buf[2..5]);
        let value = u32::from_le_bytes(buf);
        value as f32 / 100.0
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    pub fn buffer(&self) -> [u8; 8] {
        let mut result = [0; 8];
        result[2..8].copy_from_slice(&self.buf);
        result[0] = 7; // chunk length (including type)
        result[1] = 0x16; // chunk type. 0x16 means format is defined in BLE specs.
        result
    }
}

/// A data service for broadcasting a battery's remaining charge (as a percentage).
///
/// Conforms to Battery Level format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[derive(Debug, Clone, Copy)]
pub struct BatteryService {
    buf: [u8; 3],
}

impl Default for BatteryService {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryService {
    pub fn new() -> Self {
        let mut data = [0u8; 3];
        data[0..2].copy_from_slice(&BATTERY_UUID.to_le_bytes());
        Self { buf: data }
    }

    pub fn set_data(&mut self, value: u8) {
        self.buf[2] = value;
    }

    pub fn data(&self) -> u8 {
        self.buf[2]
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    pub fn buffer(&self) -> [u8; 5] {
        let mut result = [0; 5];
        result[2..5].copy_from_slice(&self.buf);
        result[0] = 4; // chunk length (including type)
        result[1] = 0x16; // chunk type. 0x16 means format is defined in BLE specs.
        result
    }
}

/// A data service for broadcasting a URL.
///
/// Conforms to specifications defined by [Google's EddyStone][eddystone] data format.
///
/// [eddystone]: https://github.com/google/eddystone
#[derive(Debug, Clone, Copy)]
pub struct UrlService {
    buf: [u8; 18],
}

impl Default for UrlService {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlService {
    const CODEX_PREFIX: [&str; 4] = ["http://www.", "https://www.", "http://", "https://"];
    const CODEX_SUFFIX: [&str; 14] = [
        ".com/", ".org/", ".edu/", ".net/", ".info/", ".biz/", ".gov/", ".com", ".org", ".edu",
        ".net", ".info", ".biz", ".gov",
    ];

    pub fn new() -> Self {
        let mut data = [0u8; 18];
        data[1] = 0x16; // chunk type 0x16 format is defined by BLE specs
        data[2..4].copy_from_slice(&EDDYSTONE_UUID.to_le_bytes());
        data[4] = 0x10; // header for embedded PA level value
        data[5] = -25i8 as u8;
        Self { buf: data }
    }

    pub fn set_data(&mut self, value: &str) {
        let mut index = 6; // index of self.buf
        let max_len = self.buf.len();
        let mut pos = 0; // position in str `value`
        let len = value.len();
        for (j, pre) in Self::CODEX_PREFIX.iter().enumerate() {
            if value[0..len].starts_with(*pre) {
                self.buf[index] = j as u8;
                pos += pre.len();
                index += 1;
                break;
            }
        }
        for (i, ch) in value.char_indices() {
            if i < pos || index >= max_len {
                continue;
            }
            for (j, post) in Self::CODEX_SUFFIX.iter().enumerate() {
                if value[i..len].starts_with(*post) {
                    self.buf[index] = j as u8;
                    pos += post.len();
                    index += 1;
                    break;
                }
            }
            if i < pos || index >= max_len {
                continue;
            }
            self.buf[index] = ch as u8;
            index += 1;
        }
        self.buf[0] = (index as u8 - 5).min(max_len as u8);
    }

    #[cfg(feature = "std")]
    pub fn data(&self) -> std::string::String {
        let mut result = std::string::String::new();
        let mut index = 0; // index of self.buf
        let max_len = self.buf[0] - 5;
        for (j, pre) in Self::CODEX_PREFIX.iter().enumerate() {
            if j as u8 == self.buf[6] {
                result.push_str(pre);
                index += 1;
                break;
            }
        }
        for (i, byte) in self.buf[6..6 + max_len as usize].iter().enumerate() {
            if index > i {
                continue;
            }
            for (j, post) in Self::CODEX_SUFFIX.iter().enumerate() {
                if j as u8 == *byte {
                    result.push_str(post);
                    index += 1;
                    break;
                }
            }
            if index > i {
                continue;
            }
            result.push(*byte as char);
            index += 1;
        }
        result
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    pub fn buffer(&self) -> &[u8] {
        let len = self.buf[0] + 1;
        &self.buf[0..len as usize]
    }
}

#[cfg(test)]
mod test {
    use super::{BatteryService, TemperatureService, UrlService};

    #[test]
    fn battery_service() {
        let mut battery = BatteryService::default();
        battery.set_data(85);
        assert_eq!(battery.data(), 85);
        assert_eq!([0x04, 0x16, 0x0F, 0x18, 0x55], battery.buffer());
    }

    #[test]
    fn temperature_service() {
        let mut temp = TemperatureService::default();
        temp.set_data(45.5);
        assert_eq!(temp.data(), 45.5);
        assert_eq!(
            [0x07, 0x16, 0x09, 0x18, 0xC6, 0x11, 0x00, 0xFE],
            temp.buffer()
        );
    }

    #[test]
    fn url_service() {
        let mut url = UrlService::default();
        url.set_data("https://www.google.com");
        assert_eq!(
            [0x0D, 0x16, 0xAA, 0xFE, 0x10, 0xE7, 0x01, 0x67, 0x6F, 0x6F, 0x67, 0x6C, 0x65, 0x07],
            *(url.buffer())
        );
    }
}

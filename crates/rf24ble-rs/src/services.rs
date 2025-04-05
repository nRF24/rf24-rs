//! A module to contain all compatible BLE services.

use crate::{
    data_manipulation::{crc24_ble, reverse_bits, whiten},
    BleChannels,
};

#[cfg(feature = "std")]
extern crate std;

/// The Temperature Service UUID number
const TEMPERATURE_UUID: u16 = 0x1809;
/// The Battery Service UUID number
const BATTERY_UUID: u16 = 0x180F;
/// The Eddystone Service UUID number
const EDDYSTONE_UUID: u16 = 0xFEAA;

/// Some common traits related to BLE service data structs.
pub mod prelude {
    /// A trait to define the factory method of constructing BLE service data from a buffer.
    pub(super) trait FromBuffer {
        fn from_buffer(buf: &[u8]) -> Self;
    }

    /// A trait to define the buffer extraction of BLE services.
    pub trait AsBuffer {
        fn buffer(&self) -> &[u8];
    }

    /// A trait to define the setter and getter of data for BLE services.
    pub trait ServiceData<T> {
        fn set_data(&mut self, value: T);
        fn data(&self) -> T;
    }
}

/// A data service that broadcasts a temperature (in Celsius)
///
/// Conforms to the Health Thermometer Measurement format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[derive(Debug, Clone, Copy)]
pub struct TemperatureService {
    buf: [u8; 8],
}

impl Default for TemperatureService {
    fn default() -> Self {
        Self::new()
    }
}

impl TemperatureService {
    /// Create an instance of [`TemperatureService`]
    pub fn new() -> Self {
        let mut data = [0u8; 8];
        data[0] = 7; // chunk length (including type)
        data[1] = 0x16; // chunk type. 0x16 means format is defined in BLE specs.
        data[2..4].copy_from_slice(&TEMPERATURE_UUID.to_le_bytes());
        data[7] = 0xFE;
        Self { buf: data }
    }
}

impl prelude::ServiceData<f32> for TemperatureService {
    /// Set the temperature measurement (in Celsius) data.
    fn set_data(&mut self, value: f32) {
        let buf = ((value * 100.0) as u32 & 0xFFFFFF).to_le_bytes();
        self.buf[4..7].copy_from_slice(&buf[0..3]);
    }

    /// Get the temperature measurement (in Celsius) data.
    fn data(&self) -> f32 {
        let mut buf = [0u8; 4];
        buf[0..3].copy_from_slice(&self.buf[4..7]);
        let value = u32::from_le_bytes(buf);
        value as f32 / 100.0
    }
}

impl prelude::AsBuffer for TemperatureService {
    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    fn buffer(&self) -> &[u8] {
        &self.buf
    }
}

impl prelude::FromBuffer for TemperatureService {
    fn from_buffer(buf: &[u8]) -> Self {
        let mut self_buf = [0u8; 8];
        self_buf.copy_from_slice(&buf[0..buf.len().min(8)]);
        Self { buf: self_buf }
    }
}

/// A data service for broadcasting a battery's remaining charge (as a percentage).
///
/// Conforms to Battery Level format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[derive(Debug, Clone, Copy)]
pub struct BatteryService {
    buf: [u8; 5],
}

impl Default for BatteryService {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryService {
    /// Create an instance of [`BatteryService`].
    pub fn new() -> Self {
        let mut data = [0u8; 5];
        data[0] = 4; // chunk length (including type)
        data[1] = 0x16; // chunk type. 0x16 means format is defined in BLE specs.
        data[2..4].copy_from_slice(&BATTERY_UUID.to_le_bytes());
        Self { buf: data }
    }
}

impl prelude::ServiceData<u8> for BatteryService {
    /// Set the battery charge level (as integer percentage) data.
    fn set_data(&mut self, value: u8) {
        self.buf[4] = value;
    }

    /// Get the battery charge level (as integer percentage) data.
    fn data(&self) -> u8 {
        self.buf[4]
    }
}

impl prelude::AsBuffer for BatteryService {
    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    fn buffer(&self) -> &[u8] {
        &self.buf
    }
}

impl prelude::FromBuffer for BatteryService {
    fn from_buffer(buf: &[u8]) -> Self {
        let mut self_buf = [0u8; 5];
        self_buf.copy_from_slice(&buf[0..buf.len().min(5)]);
        Self { buf: self_buf }
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

    /// Create an instance of [`UrlService`].
    pub fn new() -> Self {
        let mut data = [0u8; 18];
        data[1] = 0x16; // chunk type 0x16 format is defined by BLE specs
        data[2..4].copy_from_slice(&EDDYSTONE_UUID.to_le_bytes());
        data[4] = 0x10; // header for embedded PA level value
        data[5] = -25i8 as u8;
        Self { buf: data }
    }

    /// Set the predicted PA (Power Amplitude) level at 1 meter radius.
    pub fn set_pa_level(&mut self, level: i8) {
        self.buf[5] = level as u8;
    }

    /// Get the predicted PA (Power Amplitude) level at 1 meter radius.
    pub fn pa_level(&self) -> i8 {
        self.buf[5] as i8
    }

    /// Set the URL to be broadcasted.
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
            if index >= max_len {
                break;
            }
            if i < pos {
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
            if i < pos {
                continue;
            }
            self.buf[index] = ch as u8;
            index += 1;
            pos += 1;
        }
        self.buf[0] = index as u8 - 1;
    }

    /// Get the URL to be broadcasted.
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
}

impl prelude::AsBuffer for UrlService {
    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    fn buffer(&self) -> &[u8] {
        let len = self.buf[0] + 1;
        &self.buf[0..len as usize]
    }
}

impl prelude::FromBuffer for UrlService {
    fn from_buffer(buf: &[u8]) -> Self {
        let max_len = buf.len().min(18);
        let mut self_buf = [0u8; 18];
        self_buf[0..max_len].copy_from_slice(&buf[0..max_len]);
        Self { buf: self_buf }
    }
}

/// A structure to represent received BLE data.
pub struct BlePayload {
    pub mac_address: [u8; 6],
    pub short_name: Option<[u8; 10]>,
    pub tx_power: Option<i8>,
    pub battery_charge: Option<BatteryService>,
    pub url: Option<UrlService>,
    pub temperature: Option<TemperatureService>,
}

impl BlePayload {
    pub fn from_bytes(buf: &mut [u8], channel: u8) -> Option<Self> {
        use prelude::FromBuffer;

        reverse_bits(buf);
        let coefficient = (BleChannels::index_of(channel).unwrap_or_default() as u8 + 37) | 0x40;
        whiten(buf, coefficient);

        let len = buf[1] as usize;
        if len > 27 {
            return None;
        }
        let len = len + 2;

        let mut crc = [0u8; 3];
        crc.copy_from_slice(&buf[len..len + 3]);
        let expected = crc24_ble(&buf[0..len]);
        if crc != expected {
            return None;
        }

        let mut mac_address = [0u8; 6];
        mac_address.copy_from_slice(&buf[2..8]);

        let mut tx_power = None;
        let mut short_name = None;
        let mut battery_charge = None;
        let mut temperature = None;
        let mut url = None;

        let mut index = 8_usize;
        while index < len {
            let chunk_len = (buf[index] - 1) as usize;
            let chunk_type = buf[index + 1];
            let start = index + 2;
            let end = index + chunk_len + 2;
            match chunk_type {
                0x08 | 0x09 => {
                    let mut name = [0u8; 10];
                    let name_len = (end - start).min(10);
                    name[0..name_len].copy_from_slice(&buf[start..start + name_len]);
                    short_name = Some(name);
                }
                0x0A => {
                    tx_power = Some(buf[start] as i8);
                }
                0x16 => {
                    let mut tmp = [0u8; 2];
                    tmp.copy_from_slice(&buf[start..start + 2]);
                    let service_id = u16::from_le_bytes(tmp);
                    match service_id {
                        BATTERY_UUID => {
                            let batt = BatteryService::from_buffer(&buf[index..end]);
                            battery_charge = Some(batt);
                        }
                        TEMPERATURE_UUID => {
                            let temp = TemperatureService::from_buffer(&buf[index..end]);
                            temperature = Some(temp);
                        }
                        EDDYSTONE_UUID => {
                            let eddystone = UrlService::from_buffer(&buf[index..end]);
                            url = Some(eddystone);
                        }
                        _ => {}
                    }
                }
                _ => {
                    // unsupported chunk type
                    // TODO: save arbitrary data from chunk as a buffer
                }
            }
            index = end;
        }
        Some(Self {
            mac_address,
            short_name,
            tx_power,
            battery_charge,
            url,
            temperature,
        })
    }
}

#[cfg(test)]
mod test {
    use rf24::PaLevel;

    use super::{
        prelude::{AsBuffer, ServiceData},
        BatteryService, BlePayload, TemperatureService, UrlService,
    };
    use crate::data_manipulation::{reverse_bits, whiten};
    use crate::{BleChannels, FakeBle, BLE_CHANNEL};

    #[test]
    fn battery_service() {
        let mut battery = BatteryService::default();
        battery.set_data(85);
        assert_eq!(battery.data(), 85);
        assert_eq!([0x04, 0x16, 0x0F, 0x18, 0x55], *battery.buffer());
    }

    #[test]
    fn temperature_service() {
        let mut temp = TemperatureService::default();
        temp.set_data(45.5);
        assert_eq!(temp.data(), 45.5);
        assert_eq!(
            [0x07, 0x16, 0x09, 0x18, 0xC6, 0x11, 0x00, 0xFE],
            *temp.buffer()
        );
    }

    #[test]
    fn url_service() {
        let mut url = UrlService::default();
        url.set_data("https://www.foo.com/bar/bazz");
        url.set_pa_level(-20);
        assert_eq!(url.pa_level(), -20);
        assert_eq!(
            [
                0x11, 0x16, 0xAA, 0xFE, 0x10, 0xEC, 0x01, 0x66, 0x6F, 0x6F, 0x00, 0x62, 0x61, 0x72,
                0x2F, 0x62, 0x61, 0x7A
            ],
            *(url.buffer())
        );
    }

    #[test]
    fn rx_battery() {
        let mut service = BatteryService::default();
        service.set_data(85);

        let mut ble = FakeBle::default();
        ble.set_name("nRF24L01");
        let channel = BLE_CHANNEL[0];
        let mut payload = ble
            .make_payload(service.buffer(), Some(PaLevel::Low), channel)
            .unwrap();

        let ble_payload = BlePayload::from_bytes(&mut payload, channel).unwrap();
        assert_eq!(&ble.mac_address, &ble_payload.mac_address);
        assert_eq!(&ble_payload.short_name.unwrap(), &ble.name[2..]);
        assert_eq!(ble_payload.tx_power.unwrap(), -12);
        assert_eq!(
            ble_payload.battery_charge.unwrap().buffer(),
            service.buffer()
        );
    }

    #[test]
    fn rx_temperature() {
        let mut service = TemperatureService::default();
        service.set_data(45.5);

        let ble = FakeBle::default();
        let channel = BLE_CHANNEL[0];
        let mut payload = ble.make_payload(service.buffer(), None, channel).unwrap();

        let ble_payload = BlePayload::from_bytes(&mut payload, channel).unwrap();
        assert_eq!(&ble.mac_address, &ble_payload.mac_address);
        assert_eq!(ble_payload.temperature.unwrap().buffer(), service.buffer());
    }

    #[test]
    fn rx_url() {
        let mut service = UrlService::default();
        service.set_data("https://www.google.com");
        let buffer = service.buffer();

        let ble = FakeBle::default();
        let channel = BLE_CHANNEL[0];
        let mut payload = ble.make_payload(buffer, None, channel).unwrap();

        let ble_payload = BlePayload::from_bytes(&mut payload, channel).unwrap();
        assert_eq!(&ble.mac_address, &ble_payload.mac_address);
        for (i, byte) in ble_payload.url.unwrap().buffer().iter().enumerate() {
            assert_eq!(buffer[i], *byte);
        }
    }

    #[test]
    fn rx_too_big() {
        let channel = BLE_CHANNEL[0];
        let coefficient = (BleChannels::index_of(channel).unwrap() as u8 + 37) | 0x40;
        let mut payload = [0u8; 32];
        payload[1] = 29;
        whiten(&mut payload, coefficient);
        reverse_bits(&mut payload);
        assert!(BlePayload::from_bytes(&mut payload, channel).is_none());
    }

    #[test]
    fn rx_bad_crc() {
        let ble = FakeBle::default();
        let channel = BLE_CHANNEL[0];
        let coefficient = (BleChannels::index_of(channel).unwrap() as u8 + 37) | 0x40;

        // bad CRC
        let mut payload = ble.make_payload(&[17u8; 18], None, channel).unwrap();
        reverse_bits(&mut payload[29..32]);
        assert!(BlePayload::from_bytes(&mut payload, coefficient).is_none());
    }

    #[test]
    fn rx_unsupported_service() {
        let buffer = [4u8, 0x16, 0xFF, 0x0F, 0xFF];

        let ble = FakeBle::default();
        let channel = BLE_CHANNEL[0];
        let mut payload = ble
            .make_payload(&buffer, Some(PaLevel::Min), channel)
            .unwrap();

        let ble_payload = BlePayload::from_bytes(
            &mut payload,
            (BleChannels::index_of(channel).unwrap() as u8 + 37) | 0x40,
        )
        .unwrap();
        assert_eq!(&ble.mac_address, &ble_payload.mac_address);
        // TODO decode custom data
    }
}

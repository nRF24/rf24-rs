#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nRF24/rf24-rs/main/docs/src/images/logo-square.png"
)]
#![doc(html_favicon_url = "https://github.com/nRF24/rf24-rs/raw/main/docs/src/images/favicon.ico")]
#![doc = include_str!("../README.md")]
//! ## Limitations
//!
//! Because the nRF24L01 wasn't designed for BLE advertising, it has some limitations that users should beware.
//!
//! 1. The maximum payload length is shortened to **18** bytes (when not broadcasting a device
//!    name nor the radio's PA level). This is calculated as:
//!
//!    ```text
//!    32 (nRF24L01 maximum) - 6 (MAC address) - 5 (required flags) - 3 (CRC checksum) = 18
//!    ```
//!
//!    Use the helper function [`FakeBle::len_available()`](fn@crate::radio::FakeBle::len_available())
//!    to determine if your payload will transmit.
//! 2. The channels that BLE use are limited to the following three:
//!
//!    - 2.402 GHz
//!    - 2.426 GHz
//!    - 2.480 GHz.
//!
//!    For convenience, use [`FakeBle::hop_channel()`](fn@crate::radio::FakeBle::hop_channel())
//!    (when radio is in TX mode only) to switch between these frequencies.
//! 3. CRC length is disabled in the nRF24L01 firmware because BLE specifications require 3 bytes,
//!    and the nRF24L01 firmware can only handle a maximum of 2.
//!    Thus, we append the required 3 bytes of the calculated CRC24 into the payload.
//! 4. Address length of a BLE packet only uses 4 bytes.
//! 5. The auto-ack (automatic acknowledgment) feature of the nRF24L01 is useless
//!    when transmitting to BLE devices, thus the automatic re-transmit and custom ACK payloads
//!    features are useless because they both depend on the automatic acknowledgments feature.
//! 6. Dynamic payloads feature of the nRF24L01 isn't compatible with BLE specifications.
//! 7. BLE specifications only allow using 1 Mbps RF data rate, so that too has been hard coded.
//! 8. Only the "on data sent" (`tx_ds`) & "on data ready" (`rx_dr`) events will have
//!    an effect on the interrupt (IRQ) pin. The "on data fail" is never
//!    triggered when the auto-ack feature is disabled.

mod radio;
pub use radio::{ble_config, BleChannels, FakeBle, BLE_CHANNEL};

pub mod data_manipulation;

pub mod services;

#[cfg(test)]
mod test {
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        digital::{Mock as PinMock, Transaction as PinTransaction},
        spi::{Mock as SpiMock, Transaction as SpiTransaction},
    };
    use rf24::radio::RF24;

    /// Takes an indefinite repetition of a tuple of 2 vectors: `(expected_data, response_data)`
    /// and generates an array of `SpiTransaction`s.
    ///
    /// NOTE: This macro is only used to generate code in unit tests (for this crate only).
    #[macro_export]
    macro_rules! spi_test_expects {
        ($( ($expected:expr , $response:expr $(,)? ) , ) + ) => {
            [
                $(
                    SpiTransaction::transaction_start(),
                    SpiTransaction::transfer_in_place($expected, $response),
                    SpiTransaction::transaction_end(),
                )*
            ]
        }
    }

    /// A tuple struct to encapsulate objects used to mock [`RF24`],
    pub struct MockRadio(
        pub RF24<SpiMock<u8>, PinMock, NoopDelay>,
        pub SpiMock<u8>,
        pub PinMock,
    );

    /// Create a mock objects using the given expectations.
    ///
    /// The `spi_expectations` parameter
    pub fn mk_radio(
        ce_expectations: &[PinTransaction],
        spi_expectations: &[SpiTransaction<u8>],
    ) -> MockRadio {
        let spi = SpiMock::new(spi_expectations);
        let ce_pin = PinMock::new(ce_expectations);
        let delay_impl = NoopDelay;
        let radio = RF24::new(ce_pin.clone(), spi.clone(), delay_impl);
        MockRadio(radio, spi, ce_pin)
    }
}

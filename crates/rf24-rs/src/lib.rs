#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nRF24/rf24-rs/main/docs/src/images/logo-square.png"
)]
#![doc(html_favicon_url = "https://github.com/nRF24/rf24-rs/raw/main/docs/src/images/favicon.ico")]
#![doc = include_str!("../README.md")]
#![no_std]

mod types;
pub use types::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};
pub mod radio;

#[cfg(test)]
mod test {
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
}

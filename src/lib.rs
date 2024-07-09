#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nRF24/RF24/master/docs/sphinx/_static/Logo%20large.png"
)]
#![doc(
    html_favicon_url = "https://github.com/nRF24/RF24/raw/master/docs/sphinx/_static/new_favicon.ico"
)]

mod enums;
pub use enums::{CrcLength, DataRate, FifoState, PaLevel};
pub use radio::{
    EsbAutoAck, EsbChannel, EsbCrcLength, EsbDataRate, EsbFifo, EsbPaLevel, EsbPayloadLength,
    EsbPipe, EsbPower, EsbRadio, EsbStatus,
};
mod radio;
#[doc(inline)]
pub use radio::{Nrf24Error, RF24};

use embedded_hal::{
    delay::DelayNs,
    digital::{ErrorKind as OutputPinError, OutputPin},
    spi::{ErrorKind as SpiError, SpiDevice},
};
use rf24::radio::{
    prelude::RadioErrorType,
    Nrf24Error, RF24,
};

use crate::{
    addresses::LogicalAddress,
    prelude::{NetworkErrorType, StaticNetwork},
};

pub enum EsbNetworkError<RadioType>
where
    RadioType: RadioErrorType,
{
    RadioError(RadioType::Error),
}

pub struct EsbNetwork {
    node_address: LogicalAddress,
    next_header_id: u8,
}

impl EsbNetwork {
    pub fn new(node_address: LogicalAddress) -> Self {
        Self {
            node_address,
            next_header_id: 0,
        }
    }
}

impl<SPI, DO, DELAY> From<Nrf24Error<SpiError, OutputPinError>>
    for EsbNetworkError<RF24<SPI, DO, DELAY>>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    fn from(value: Nrf24Error<SpiError, OutputPinError>) -> Self {
        EsbNetworkError::RadioError(value)
    }
}

impl<SPI, DO, DELAY> NetworkErrorType<RF24<SPI, DO, DELAY>> for EsbNetwork
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type Error = EsbNetworkError<RF24<SPI, DO, DELAY>>;
}

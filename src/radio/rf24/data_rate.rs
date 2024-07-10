use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbDataRate, Nrf24Error, RF24};
use crate::DataRate;
use super::registers;

impl<SPI, DO, DELAY> EsbDataRate for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type DataRateErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_data_rate(&mut self) -> Result<DataRate, Self::DataRateErrorType> {
        let result = self.spi_read(1, registers::RF_SETUP);
        result?;
        let da_bin = self._buf[1] >> 3 & 3;
        match da_bin {
            0 => Ok(DataRate::Mbps1),
            1 => Ok(DataRate::Mbps2),
            2 => Ok(DataRate::Kbps250),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), Self::DataRateErrorType> {
        let da_bin = {
            match data_rate {
                DataRate::Mbps1 => {
                    self._tx_delay = 280;
                    0 as u8
                }
                DataRate::Mbps2 => {
                    self._tx_delay = 240;
                    1 as u8
                }
                DataRate::Kbps250 => {
                    self._tx_delay = 505;
                    2 as u8
                }
            }
        } << 3;
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & (3 << 3) | da_bin;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

use super::{Nrf24Error, RF24};
use crate::radio::prelude::EsbDetails;
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

#[cfg(any(feature = "defmt", feature = "std"))]
use super::{mnemonics, registers};
#[cfg(any(feature = "defmt", feature = "std"))]
use crate::{
    radio::prelude::{
        EsbChannel, EsbCrcLength, EsbDataRate, EsbFifo, EsbPaLevel, EsbPayloadLength, EsbPipe,
        EsbStatus,
    },
    StatusFlags,
};

#[cfg(feature = "std")]
extern crate std;

impl<SPI, DO, DELAY> EsbDetails for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type LoggingErrorType = Nrf24Error<SPI::Error, DO::Error>;

    #[cfg(feature = "defmt")]
    #[cfg(target_os = "none")]
    fn print_details(&mut self) -> Result<(), Self::LoggingErrorType> {
        defmt::println!("Is a plus variant_________{=bool}", self.is_plus_variant());

        let channel = self.get_channel()?;
        defmt::println!(
            "Channel___________________{=u8} ~ {=u16} Hz",
            channel,
            channel as u16 + 2400u16
        );

        defmt::println!("RF Data Rate______________{}", self.get_data_rate()?);
        defmt::println!("RF Power Amplifier________{}", self.get_pa_level()?);

        self.spi_read(1, registers::RF_SETUP)?;
        let rf_setup = self._buf[1];
        defmt::println!("RF LNA enabled____________{=bool}", rf_setup & 1 > 0);

        defmt::println!("CRC Length________________{}", self.get_crc_length()?);

        defmt::println!(
            "Address length____________{=u8} bytes",
            self.get_address_length()?
        );

        defmt::println!(
            "TX Payload lengths________{=u8} bytes",
            self.get_payload_length()?
        );

        self.spi_read(1, registers::SETUP_RETR)?;
        let retry_setup = self._buf[1];
        defmt::println!(
            "Auto retry delay__________{=u16} microseconds",
            (retry_setup >> 4) as u16 * 250 + 250
        );
        defmt::println!(
            "Auto retry attempts_______{=u8} maximum",
            retry_setup & 0x0F
        );

        self.spi_read(1, registers::FIFO_STATUS)?;
        defmt::println!(
            "Re-use TX FIFO____________{=bool}",
            (self._buf[1] & 0x80) > 0
        );

        self.spi_read(1, registers::OBSERVE_TX)?;
        let observer = self._buf[1];
        defmt::println!(
            "Packets lost\n    on current channel____{=u8}",
            observer >> 4
        );
        defmt::println!(
            "Retry attempts made\n    for last transmission_{=u8}",
            observer & 0xF
        );

        self.spi_read(1, registers::CONFIG)?;
        self._config_reg = self._buf[1];
        let mut flags = StatusFlags::default();
        self.get_status_flags(&mut flags);
        defmt::println!(
            "IRQ on Data Ready_________{=bool}",
            self._config_reg & mnemonics::MASK_RX_DR > 0
        );
        defmt::println!("    Data Ready triggered__{=bool}", flags.rx_dr);
        defmt::println!(
            "IRQ on Data Fail__________{=bool}",
            self._config_reg & mnemonics::MASK_MAX_RT > 0
        );
        defmt::println!("    Data Failed triggered_{=bool}", flags.tx_df);
        defmt::println!(
            "IRQ on Data Sent__________{=bool}",
            self._config_reg & mnemonics::MASK_TX_DS > 0
        );
        defmt::println!("    Data Sent triggered___{=bool}", flags.tx_ds);

        let fifo = self.get_fifo_state(true)?;
        defmt::println!("TX FIFO___________________{}", fifo);
        let fifo = self.get_fifo_state(false)?;
        defmt::println!("RX FIFO___________________{}", fifo);

        self.spi_read(1, registers::FEATURE)?;
        let features = self._buf[1];
        defmt::println!("Ask no ACK allowed________{=bool}", features & 1 > 0);
        defmt::println!("ACK Payload enabled_______{=bool}", features & 2 > 0);

        self.spi_read(1, registers::DYNPD)?;
        defmt::println!("Dynamic Payloads__________0b{=0..8}", self._buf[1]);

        self.spi_read(1, registers::EN_AA)?;
        defmt::println!("Auto Acknowledgment_______0b{=0..8}", self._buf[1]);

        defmt::println!(
            "Primary Mode______________{}X",
            if self._config_reg & 1 > 0 { "R" } else { "T" }
        );
        defmt::println!(
            "Powered Up________________{=bool}",
            self._config_reg & 2 > 0
        );
        Ok(())
    }

    #[cfg(not(any(feature = "defmt", feature = "std")))]
    fn print_details(&mut self) -> Result<(), Self::LoggingErrorType> {
        Ok(())
    }

    #[cfg(not(target_os = "none"))]
    #[cfg(feature = "std")]
    fn print_details(&mut self) -> Result<(), Self::LoggingErrorType> {
        std::println!("Is a plus variant_________{}", self.is_plus_variant());

        let channel = self.get_channel()?;
        std::println!(
            "Channel___________________{channel} ~ {} Hz",
            channel as u16 + 2400u16
        );

        std::println!("RF Data Rate______________{}", self.get_data_rate()?);
        std::println!("RF Power Amplifier________{}", self.get_pa_level()?);

        self.spi_read(1, registers::RF_SETUP)?;
        let rf_setup = self._buf[1];
        std::println!("RF LNA enabled____________{}", rf_setup & 1 > 0);

        std::println!("CRC Length________________{}", self.get_crc_length()?);

        std::println!(
            "Address length____________{} bytes",
            self.get_address_length()?
        );

        std::println!(
            "TX Payload lengths________{} bytes",
            self.get_payload_length()?
        );

        self.spi_read(1, registers::SETUP_RETR)?;
        let retry_setup = self._buf[1];
        std::println!(
            "Auto retry delay__________{} microseconds",
            (retry_setup >> 4) as u16 * 250 + 250
        );
        std::println!("Auto retry attempts_______{} maximum", retry_setup & 0x0F);

        self.spi_read(1, registers::FIFO_STATUS)?;
        std::println!("Re-use TX FIFO____________{}", (self._buf[1] & 0x80) > 0);

        self.spi_read(1, registers::OBSERVE_TX)?;
        let observer = self._buf[1];
        std::println!("Packets lost\n    on current channel____{}", observer >> 4);
        std::println!(
            "Retry attempts made\n    for last transmission_{}",
            observer & 0xF
        );

        self.spi_read(1, registers::CONFIG)?;
        self._config_reg = self._buf[1];
        let mut flags = StatusFlags::default();
        self.get_status_flags(&mut flags);
        std::println!(
            "IRQ on Data Ready_________{}",
            self._config_reg & mnemonics::MASK_RX_DR > 0
        );
        std::println!("    Data Ready triggered__{}", flags.rx_dr);
        std::println!(
            "IRQ on Data Fail__________{}",
            self._config_reg & mnemonics::MASK_MAX_RT > 0
        );
        std::println!("    Data Failed triggered_{}", flags.tx_df);
        std::println!(
            "IRQ on Data Sent__________{}",
            self._config_reg & mnemonics::MASK_TX_DS > 0
        );
        std::println!("    Data Sent triggered___{}", flags.tx_ds);

        let fifo = self.get_fifo_state(true)?;
        std::println!("TX FIFO___________________{}", fifo);
        let fifo = self.get_fifo_state(false)?;
        std::println!("RX FIFO___________________{}", fifo);

        self.spi_read(1, registers::FEATURE)?;
        let features = self._buf[1];
        std::println!("Ask no ACK allowed________{}", features & 1 > 0);
        std::println!("ACK Payload enabled_______{}", features & 2 > 0);

        self.spi_read(1, registers::DYNPD)?;
        std::println!("Dynamic Payloads__________{:#010b}", self._buf[1]);

        self.spi_read(1, registers::EN_AA)?;
        std::println!("Auto Acknowledgment_______{:#010b}", self._buf[1]);

        std::println!(
            "Primary Mode______________{}X",
            if self._config_reg & 1 > 0 { "R" } else { "T" }
        );
        std::println!("Powered Up________________{}", self._config_reg & 2 > 0);
        Ok(())
    }
}

use bitfield_struct::bitfield;

use crate::{CrcLength, DataRate, PaLevel};

use super::mnemonics;

#[bitfield(u8, order = Msb)]
pub(crate) struct Config {
    #[bits(1)]
    _padding: u8,

    /// Configure the radio's "RX Data Ready" IRQ event.
    #[bits(1, access = None)]
    pub rx_dr: bool,

    /// Configure the radio's "TX Data Sent" IRQ event.
    #[bits(1, access = None)]
    pub tx_ds: bool,

    /// Configure the radio's "TX Data Fail" IRQ event.
    #[bits(1, access = None)]
    pub tx_df: bool,

    #[bits(2, access = None, default = 3)]
    pub crc_length: u8,

    pub power: bool,

    pub is_rx: bool,
}

impl Config {
    pub(crate) const CRC_MASK: u8 = 0b1100;

    pub const fn crc_length(&self) -> CrcLength {
        CrcLength::from_bits(self.into_bits() & Self::CRC_MASK)
    }

    pub fn with_crc_length(self, length: CrcLength) -> Self {
        let new_val = self.into_bits() & !Self::CRC_MASK | length.into_bits();
        Self::from_bits(new_val)
    }

    pub const fn rx_dr(&self) -> bool {
        (self.into_bits() & mnemonics::MASK_RX_DR) == 0
    }

    pub fn with_rx_dr(self, enable: bool) -> Self {
        Self::from_bits(
            self.into_bits() & !mnemonics::MASK_RX_DR | ((!enable as u8) * mnemonics::MASK_RX_DR),
        )
    }

    pub const fn tx_ds(&self) -> bool {
        (self.into_bits() & mnemonics::MASK_TX_DS) == 0
    }

    pub fn with_tx_ds(self, enable: bool) -> Self {
        Self::from_bits(
            self.into_bits() & !mnemonics::MASK_TX_DS | ((!enable as u8) * mnemonics::MASK_TX_DS),
        )
    }

    pub const fn tx_df(&self) -> bool {
        (self.into_bits() & mnemonics::MASK_MAX_RT) == 0
    }

    pub fn with_tx_df(self, enable: bool) -> Self {
        Self::from_bits(
            self.into_bits() & !mnemonics::MASK_MAX_RT | ((!enable as u8) * mnemonics::MASK_MAX_RT),
        )
    }

    pub fn as_rx(self) -> Self {
        Self::from_bits(self.into_bits() | 1)
    }

    pub fn as_tx(self) -> Self {
        Self::from_bits(self.into_bits() & !1)
    }
}

#[bitfield(u8, order = Msb)]
pub(crate) struct SetupRetry {
    /// The auto-retry feature's `delay`.
    #[bits(4, default = 5)]
    pub ard: u8,

    /// The auto-retry feature's `count`.
    #[bits(4, default = 15)]
    pub arc: u8,
}

#[bitfield(u8, order = Msb)]
pub(crate) struct SetupRfAw {
    #[bits(2, access = None, default = 3)]
    address_length: u8,

    #[bits(3, access = None)]
    data_rate: u8,

    #[bits(2, access = None, default = 3)]
    pa_level: u8,

    #[bits(1, default = true)]
    pub lna_enable: bool,
}

impl SetupRfAw {
    const PA_MASK: u8 = 0b110;
    const DATA_RATE_MASK: u8 = 0x28;
    const ADDR_OFFSET: u8 = 6;

    pub const fn address_length(&self) -> u8 {
        (self.into_bits() >> Self::ADDR_OFFSET) + 2
    }

    pub fn with_address_length(self, length: u8) -> Self {
        let new_val = self.into_bits() & !(0b11 << Self::ADDR_OFFSET);
        Self::from_bits(new_val | ((length.clamp(2, 5) - 2) << Self::ADDR_OFFSET))
    }

    pub const fn data_rate(&self) -> DataRate {
        DataRate::from_bits(self.into_bits() & Self::DATA_RATE_MASK)
    }

    pub fn with_data_rate(self, data_rate: DataRate) -> Self {
        let new_val = self.into_bits() & !Self::DATA_RATE_MASK;
        Self::from_bits(new_val | data_rate.into_bits())
    }

    pub const fn pa_level(&self) -> PaLevel {
        PaLevel::from_bits(self.into_bits() & Self::PA_MASK)
    }

    pub fn with_pa_level(self, level: PaLevel) -> Self {
        let new_val = self.into_bits() & !Self::PA_MASK;
        Self::from_bits(new_val | level.into_bits())
    }
}

#[bitfield(u8, order = Msb)]
pub(crate) struct Feature {
    #[bits(3, access = WO)]
    pub address_length: u8,

    #[bits(1)]
    _padding: u8,

    pub is_plus_variant: bool,

    #[bits(1, access = RO)]
    pub dynamic_payloads: bool,

    #[bits(1, access = RO)]
    pub ack_payloads: bool,

    pub ask_no_ack: bool,
}

impl Feature {
    pub const REG_MASK: u8 = 7;

    pub const fn address_length(&self) -> u8 {
        self.into_bits() >> Self::ADDRESS_LENGTH_OFFSET
    }

    pub fn with_dynamic_payloads(self, enable: bool) -> Self {
        let mut new_val = self.into_bits() & !(1u8 << Self::DYNAMIC_PAYLOADS_OFFSET);
        if !enable {
            // disable ACK payloads also
            new_val &= !(1u8 << Self::ACK_PAYLOADS_OFFSET);
        } else {
            new_val |= 1u8 << Self::DYNAMIC_PAYLOADS_OFFSET;
        }
        Self::from_bits(new_val)
    }

    pub fn with_ack_payloads(self, enable: bool) -> Self {
        let mut new_value = self.into_bits() & !(1u8 << Self::ACK_PAYLOADS_OFFSET);
        if enable {
            // enable dynamic payloads also
            new_value |= (1u8 << Self::ACK_PAYLOADS_OFFSET) | (1u8 << Self::DYNAMIC_PAYLOADS_OFFSET)
        }
        Self::from_bits(new_value)
    }
}

// unit tests found in crate::radio::config::test

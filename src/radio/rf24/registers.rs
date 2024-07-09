///! A private module encapsulating register offsets for the nRF24L01.

pub const CONFIG: u8 = 0x00;
pub const EN_AA: u8 = 0x01;
pub const EN_RXADDR: u8 = 0x02;
pub const SETUP_AW: u8 = 0x03;
pub const SETUP_RETR: u8 = 0x04;
pub const RF_CH: u8 = 0x05;
pub const RF_SETUP: u8 = 0x06;
pub const STATUS: u8 = 0x07;
pub const OBSERVE_TX: u8 = 0x08;
pub const RPD: u8 = 0x09;
pub const RX_ADDR_P0: u8 = 0x0A;
pub const TX_ADDR: u8 = 0x10;
pub const RX_PW_P0: u8 = 0x11;
pub const FIFO_STATUS: u8 = 0x17;
pub const DYNPD: u8 = 0x1C;
pub const FEATURE: u8 = 0x1D;

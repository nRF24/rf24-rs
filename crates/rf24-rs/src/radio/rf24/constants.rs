/// A private module encapsulating register offsets for the nRF24L01.
pub mod registers {
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
}

/// A private module encapsulating SPI commands for the nRF24L01.
pub mod commands {
    pub const W_REGISTER: u8 = 0x20;
    pub const ACTIVATE: u8 = 0x50;
    pub const R_RX_PL_WID: u8 = 0x60;
    pub const R_RX_PAYLOAD: u8 = 0x61;
    pub const W_TX_PAYLOAD: u8 = 0xA0;
    pub const W_TX_PAYLOAD_NO_ACK: u8 = 0xB0;
    pub const W_ACK_PAYLOAD: u8 = 0xA8;
    pub const FLUSH_TX: u8 = 0xE1;
    pub const FLUSH_RX: u8 = 0xE2;
    pub const REUSE_TX_PL: u8 = 0xE3;
    pub const NOP: u8 = 0xFF;
}

/// A private module to encapsulate bit mnemonics
pub mod mnemonics {
    pub const MASK_RX_DR: u8 = 1 << 6;
    pub const MASK_TX_DS: u8 = 1 << 5;
    pub const MASK_MAX_RT: u8 = 1 << 4;
}

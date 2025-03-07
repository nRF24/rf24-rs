//! This example demonstrates how to quickly and easily
//! change the radio's configuration.
//!
//! This example requires no counterpart as
//! it does not actually transmit nor receive anything.
//!
//! See documentation at https://docs.rs/rf24-rs
#![no_std]

use anyhow::Result;

use rf24::{
    radio::{prelude::*, EsbConfig, RF24},
    CrcLength,
};
use rf24_rs_examples::debug_err;
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    println, BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl, SpidevDevice as SpiImpl,
};

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
}

impl App {
    pub fn new() -> Result<Self> {
        // instantiate a hardware peripherals on the board
        let mut board = BoardHardware::default()?;

        // instantiate radio object using board's hardware
        let radio = RF24::new(
            board.default_ce_pin()?,
            BoardHardware::default_spi_device()?,
            DelayImpl,
        );
        Ok(Self { board, radio })
    }

    /// Setup the radio for this example.
    ///
    /// This will initialize and configure the [`App::radio`] object.
    pub fn setup(&mut self) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(debug_err)?;
        Ok(())
    }

    /// Configure the radio for 2 different scenarios and
    /// print the configuration details for each.
    pub fn run(&mut self) -> Result<()> {
        let normal_context = EsbConfig::default()
            .with_rx_address(1, b"1Node")
            .with_tx_address(b"2Node");

        let ble_addr = [0x71, 0x91, 0x7d, 0x6b];
        let ble_context = EsbConfig::default()
            .with_channel(2) // BLE specs hop/rotate amongst channels 2, 26, and 80
            .with_crc_length(CrcLength::Disabled)
            .with_auto_ack(0)
            .with_address_length(4)
            .with_rx_address(1, &ble_addr)
            .with_tx_address(&ble_addr);

        println!("Settings for BLE context\n------------------------");
        self.radio.with_config(&ble_context).map_err(debug_err)?;
        self.radio.print_details().map_err(debug_err)?;

        println!("\nSettings for normal context\n---------------------------");
        self.radio.with_config(&normal_context).map_err(debug_err)?;
        self.radio.print_details().map_err(debug_err)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.setup()?;
    app.run()?;
    app.radio.power_down().map_err(debug_err)?;
    Ok(())
}

//! The simplest example of using the nRF24L01 transceiver to send and receive.
//!
//! This example is meant to be run on 2 separate nRF24L01 transceivers.
//!
//! See documentation at <https://docs.rs/rf24-rs>
#![no_std]

use anyhow::Result;
use core::{f32, time::Duration};
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    PaLevel,
};
use rf24_rs_examples::debug_err;
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    print, println, BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl,
    SpidevDevice as SpiImpl,
};

#[cfg(feature = "linux")]
extern crate std;
#[cfg(feature = "linux")]
use std::{
    io::{stdin, stdout, Write},
    string::{String, ToString},
    time::Instant,
};

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
    /// We will be using a 32-bit float value (little-endian) as our payload.
    payload: f32,
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
        Ok(Self {
            board,
            radio,
            payload: 0.0,
        })
    }

    /// Setup the radio for this example.
    ///
    /// This will initialize and configure the [`App::radio`] object.
    pub fn setup(&mut self, radio_number: u8) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(debug_err)?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio.set_pa_level(PaLevel::Low).map_err(debug_err)?;

        // we'll be using a 32-bit float, so set the payload length to 4 bytes
        self.radio.set_payload_length(4).map_err(debug_err)?;

        let address = [b"1Node", b"2Node"];

        // set TX address of RX node (always uses pipe 0)
        self.radio
            .as_tx(Some(address[radio_number as usize])) // enter inactive TX mode
            .map_err(debug_err)?;

        // set RX address of TX node into an RX pipe
        self.radio
            .open_rx_pipe(1, address[1 - radio_number as usize]) // using pipe 1
            .map_err(debug_err)?;
        Ok(())
    }

    /// The TX role.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    pub fn tx(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.as_tx(None).map_err(debug_err)?;

        let mut remaining = count;
        while remaining > 0 {
            let buf = self.payload.to_le_bytes();
            let start = Instant::now();
            let result = self.radio.send(&buf, false).map_err(debug_err)?;
            let end = Instant::now();
            if result {
                // succeeded
                println!(
                    "Transmission successful! Time to Transmit: {} us. Sent: {}",
                    end.saturating_duration_since(start).as_micros(),
                    self.payload
                );
                self.payload += 0.01;
            } else {
                // failed
                println!("Transmission failed or timed out");
            }
            remaining -= 1;
            DelayImpl.delay_ms(1000);
        }

        // recommended behavior is to keep in TX mode while idle
        self.radio.as_tx(None).map_err(debug_err)?; // put the radio into inactive TX mode

        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // put radio into active RX mode
        self.radio.as_rx().map_err(debug_err)?;
        let mut end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time {
            let mut pipe = 15u8;
            if self.radio.available_pipe(&mut pipe).map_err(debug_err)? {
                let mut buf = [0u8; 4];
                let len = self.radio.read(&mut buf, None).map_err(debug_err)?;
                self.payload = f32::from_le_bytes(buf);
                // print pipe number and payload length and payload
                println!("Received {len} bytes on pipe {pipe}: {}", self.payload);
                // reset timeout
                end_time = Instant::now() + Duration::from_secs(timeout as u64);
            }
        }

        // recommended behavior is to keep in TX mode while idle
        self.radio.as_tx(None).map_err(debug_err)?; // put the radio into inactive TX mode

        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'R' for receiver role.\n\
        *** Enter 'T' for transmitter role.\n\
        *** Enter 'Q' to quit example.";
        println!("{prompt}");
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let mut inputs = input.trim().split(' ');
        let role = inputs
            .next()
            .map(|v| v.to_uppercase())
            .unwrap_or("?".to_string());
        if role.starts_with('T') {
            let count = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(5);
            self.tx(count)?;
            return Ok(true);
        } else if role.starts_with('R') {
            let timeout = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(6);
            self.rx(timeout)?;
            return Ok(true);
        } else if role.starts_with('Q') {
            self.radio.power_down().map_err(debug_err)?;
            return Ok(false);
        }
        println!("{role} is an unrecognized input. Please try again.");
        Ok(true)
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    let mut input = String::new();
    print!("Which radio is this? Enter '0' or '1'. Defaults to '0' ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let radio_number = input
        .trim()
        .chars()
        .next()
        .map(|c| if c == '1' { 1 } else { 0 })
        .unwrap_or_default();
    app.setup(radio_number)?;
    while app.set_role()? {}
    Ok(())
}

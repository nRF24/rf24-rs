//! This example uses 1 nRF24L01 to receive data from up to 6 other
//! transceivers. This technique is called "multiceiver" in the datasheet.
//!
//! This example is meant to be run on at least 2 separate nRF24L01 transceivers.
//! Although, this example can be used on 7 transceivers at most simultaneously.
//!
//! See documentation at https://docs.rs/rf24-rs
#![no_std]

use anyhow::Result;
use core::time::Duration;
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    PaLevel,
};
use rf24_rs_examples::debug_err;
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    println, BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl, SpidevDevice as SpiImpl,
};

#[cfg(feature = "linux")]
extern crate std;
#[cfg(feature = "linux")]
use std::{
    io::stdin,
    string::{String, ToString},
    time::Instant,
};

/// The payloads for this example will consist of 2 32-bit integers (8 bytes)
const SIZE: u8 = 8;

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
    /// The addresses for all transmitting nRF24L01 nodes
    addresses: [[u8; 5]; 6],
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
            addresses: [
                [0x78, 0x78, 0x78, 0x78, 0x78],
                [0xf1, 0xb6, 0xb5, 0xb4, 0xb3],
                [0xcd, 0xb6, 0xb5, 0xb4, 0xb3],
                [0xa3, 0xb6, 0xb5, 0xb4, 0xb3],
                [0x0f, 0xb6, 0xb5, 0xb4, 0xb3],
                [0x05, 0xb6, 0xb5, 0xb4, 0xb3],
            ],
        })
    }

    /// Setup the radio for this example.
    ///
    /// This will initialize and configure the [`App::radio`] object.
    pub fn setup(&mut self) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(debug_err)?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio.set_pa_level(PaLevel::Low).map_err(debug_err)?;

        // set the payload length to 8 bytes
        self.radio.set_payload_length(SIZE).map_err(debug_err)?;

        Ok(())
    }

    /// Start transmitting to the base station.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    pub fn tx(&mut self, node_number: u8, count: u8) -> Result<()> {
        // According to the datasheet, the auto-retry features's delay value should
        // be "skewed" to allow the RX node to receive 1 transmission at a time.
        // So, use varying delay between retry attempts and 15 (at most) retry attempts
        self.radio
            .set_auto_retries(((node_number * 3) % 12) + 3, 15) // max value is 15 for both args
            .map_err(debug_err)?;

        // put radio into TX mode
        self.radio.as_tx().map_err(debug_err)?;
        // set the TX address to the address of the base station.
        self.radio
            .open_tx_pipe(&self.addresses[node_number as usize])
            .map_err(debug_err)?;
        let mut counter = 0;
        while counter < count {
            counter += 1;
            let mut payload = [0u8; SIZE as usize];
            payload[0..4].copy_from_slice(&(node_number as u32).to_le_bytes());
            payload[4..SIZE as usize].copy_from_slice(&(counter as u32).to_le_bytes());
            let start = Instant::now();
            let result = self.radio.send(&payload, false).map_err(debug_err)?;
            let end = Instant::now();
            if result {
                // succeeded
                println!(
                    "Transmission of payloadID {counter} as node {node_number} successful! Transmission time: {} us",
                    end.saturating_duration_since(start).as_micros(),
                );
            } else {
                // failed
                println!("Transmission failed or timed out");
            }
            DelayImpl.delay_ms(1000);
        }
        Ok(())
    }

    /// Use the [`App::radio`] as a base station for listening to all nodes.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // write the addresses to all pipes.
        for (pipe, addr) in self.addresses.iter().enumerate() {
            self.radio
                .open_rx_pipe(pipe as u8, addr)
                .map_err(debug_err)?;
        }
        // put radio into active RX mode
        self.radio.as_rx().map_err(debug_err)?;
        let mut end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time {
            let mut pipe = 15u8;
            if self.radio.available_pipe(&mut pipe).map_err(debug_err)? {
                let mut buf = [0u8; SIZE as usize];
                let len = self.radio.read(&mut buf, None).map_err(debug_err)?;
                let node = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let payload_id = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                // print pipe number and payload length and payload
                println!(
                    "Received {len} bytes on pipe {pipe} from node {node}. PayloadID: {payload_id}"
                );
                // reset timeout
                end_time = Instant::now() + Duration::from_secs(timeout as u64);
            }
        }

        // It is highly recommended to keep the radio idling in an inactive TX mode
        self.radio.as_tx().map_err(debug_err)?;
        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'R' for receiver role.\n\
        *** Enter 'T' for transmitter role.\n    \
        Use 'T n' to transmit as node n; n must be in range [0, 5].\n\
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
            let node_number = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or_default();
            let count = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(5);
            self.tx(node_number, count)?;
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
    app.setup()?;
    while app.set_role()? {}
    Ok(())
}

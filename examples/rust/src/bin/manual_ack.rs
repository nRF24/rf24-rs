//! This example uses the nRF24L01 to transmit and respond with an
//! acknowledgment (ACK) transmissions. Notice that the auto-ack feature is
//! enabled, but this example doesn't use automatic ACK payloads because automatic
//! ACK payloads' data will always be outdated by 1 transmission. Instead, this
//! example uses a call-and-response paradigm.
//!
//! This example is meant to be run on 2 separate nRF24L01 transceivers.
//!
//! See documentation at <https://docs.rs/rf24-rs>
#![no_std]

use anyhow::Result;
use core::time::Duration;
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    PaLevel, StatusFlags,
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
    borrow::ToOwned,
    io::{stdin, Write},
    string::{String, ToString},
    time::Instant,
};

// we'll be using a a 7-byte string with a 1-byte counter as a response payload.
const SIZE: u8 = 8;

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
    /// We will be using a incrementing integer value as part of our payloads.
    counter: u8,
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
            counter: 0,
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

        // set static payload length
        self.radio.set_payload_length(SIZE).map_err(debug_err)?;

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

        // declare our outgoing payload.
        // `\x00` is null terminator for the string portion.
        // `0` is placeholder for the u16 counter.
        let mut outgoing_payload = b"Hello \x000".to_owned();
        let mut remaining = count;
        while remaining > 0 {
            outgoing_payload[7] = self.counter;
            let start = Instant::now();
            let result = self
                .radio
                .send(&outgoing_payload, false)
                .map_err(debug_err)?;
            let mut got_response = false;
            if result {
                // send successful. now wait for a response
                self.radio.as_rx().map_err(debug_err)?;
                let response_timeout = Instant::now() + Duration::from_millis(150);
                while Instant::now() < response_timeout && !got_response {
                    got_response = self.radio.available().map_err(debug_err)?;
                }
                self.radio.as_tx(None).map_err(debug_err)?;
            }
            let end = Instant::now();

            // print results
            if result {
                print!(
                    "Transmission successful! Time to Transmit: {} us. Sent: {}{} ",
                    end.saturating_duration_since(start).as_micros(),
                    String::from_utf8_lossy(&outgoing_payload[0..6]),
                    self.counter,
                );
                if got_response {
                    let mut response = [0u8; SIZE as usize];
                    self.radio.read(&mut response, None).map_err(debug_err)?;
                    self.counter = response[7];
                    println!(
                        "Received: {}{}",
                        String::from_utf8_lossy(&response[0..6]),
                        self.counter,
                    );
                } else {
                    println!("No response received.");
                }
                self.counter += 1;
            } else {
                println!("Transmission failed or timed out");
            }
            remaining -= 1;
            DelayImpl.delay_ms(1000);
        }

        // recommended behavior is to keep in TX mode while idle
        self.radio.as_tx(None).map_err(debug_err)?; // enter inactive TX mode

        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // put radio into active RX mode
        self.radio.as_rx().map_err(debug_err)?;

        // declare our outgoing payload
        // `\x00` is the null terminator for the string portion
        // `0` is the 2-byte placeholder for our counter value
        let mut outgoing_payload = b"World \x000".to_owned();

        let mut end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time {
            let mut pipe = 15u8;
            if self.radio.available_pipe(&mut pipe).map_err(debug_err)? {
                // received a payload
                let mut incoming_payload = [0u8; SIZE as usize];
                let len = self
                    .radio
                    .read(&mut incoming_payload, None)
                    .map_err(debug_err)?;
                self.counter = incoming_payload[7];
                outgoing_payload[7] = self.counter;

                // send a response
                self.radio.as_tx(None).map_err(debug_err)?;
                let mut response_result = false;
                let mut flags = StatusFlags::default();
                self.radio
                    .write(&outgoing_payload, false, true)
                    .map_err(debug_err)?;
                let response_timeout = Instant::now() + Duration::from_millis(150);
                while Instant::now() < response_timeout && !response_result {
                    self.radio.update().map_err(debug_err)?;
                    self.radio.get_status_flags(&mut flags);
                    if flags.tx_ds() {
                        response_result = true;
                    } else if flags.tx_df() {
                        self.radio.rewrite().map_err(debug_err)?;
                    }
                }
                self.radio.as_rx().map_err(debug_err)?;

                // print pipe number and payload length and payload
                print!(
                    "Received {len} bytes on pipe {pipe}: {}{} ",
                    String::from_utf8_lossy(&incoming_payload[0..6]),
                    self.counter,
                );
                if response_result {
                    println!(
                        "Sent: {}{}",
                        String::from_utf8_lossy(&outgoing_payload[0..6]),
                        self.counter,
                    );
                } else {
                    println!("Response failed or timed out");
                }

                // reset timeout
                end_time = Instant::now() + Duration::from_secs(timeout as u64);

                // increment counter
                self.counter += 1;
            }
        }

        // recommended behavior is to keep in TX mode while idle
        self.radio.as_tx(None).map_err(debug_err)?; // enter inactive TX mode

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
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
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

#![no_std]

use core::time::Duration;

use anyhow::{anyhow, Result};
use embedded_hal::delay::DelayNs;
use rf24::{
    radio::{prelude::*, RF24},
    PaLevel,
};
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
    io::Write,
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
        self.radio.init().map_err(|e| anyhow!("{e:?}"))?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio
            .set_pa_level(PaLevel::Low)
            .map_err(|e| anyhow!("{e:?}"))?;

        // we'll be using a automatic ACK payloads, so enable the dynamic payload length feature
        self.radio
            .set_dynamic_payloads(true)
            .map_err(|e| anyhow!("{e:?}"))?;
        // enable ACK packet payloads
        self.radio
            .set_ack_payloads(true)
            .map_err(|e| anyhow!("{e:?}"))?;

        let address = [b"1Node", b"2Node"];
        self.radio
            .open_tx_pipe(address[radio_number as usize])
            .map_err(|e| anyhow!("{e:?}"))?;
        self.radio
            .open_rx_pipe(1, address[1 - radio_number as usize])
            .map_err(|e| anyhow!("{e:?}"))?;
        Ok(())
    }

    /// The TX role.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    pub fn tx(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.as_tx().map_err(|e| anyhow!("{e:?}"))?;

        // declare our outgoing payload.
        // `\x00` is null terminator for the string portion.
        // `0` is placeholder for our counter value.
        let mut outgoing_payload = b"Hello \x000".to_owned();
        let mut remaining = count;
        while remaining > 0 {
            outgoing_payload[7] = self.counter;
            let start = Instant::now();
            let result = self
                .radio
                .send(&outgoing_payload, false)
                .map_err(|e| anyhow!("{e:?}"))?;
            let end = Instant::now();
            if result {
                // succeeded
                print!(
                    "Transmission successful! Time to Transmit: {} us. Sent: {}{} Received: ",
                    end.saturating_duration_since(start).as_micros(),
                    String::from_utf8_lossy(&outgoing_payload[0..6]),
                    self.counter,
                );
                self.counter += 1;
                if self.radio.available().map_err(|e| anyhow!("{e:?}"))? {
                    let mut incoming_payload = [0u8; 8];
                    self.radio
                        .read(&mut incoming_payload, None)
                        .map_err(|e| anyhow!("{e:?}"))?;
                    println!(
                        "{}{}",
                        String::from_utf8_lossy(&incoming_payload[0..6]),
                        incoming_payload[7],
                    );
                } else {
                    println!("An empty ACK payload");
                }
            } else {
                // failed
                println!("Transmission failed or timed out");
            }
            remaining -= 1;
            DelayImpl.delay_ms(1000);
        }
        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // put radio into active RX mode
        self.radio.as_rx().map_err(|e| anyhow!("{e:?}"))?;

        // declare our outgoing payload
        // `\x00` is the null terminator for the string portion
        // `0` is the placeholder for our counter value
        let mut outgoing_payload = b"World \x000".to_owned();
        outgoing_payload[7] = self.counter;
        // load ACK for first response
        self.radio
            .write_ack_payload(1, &outgoing_payload)
            .map_err(|e| anyhow!("{e:?}"))?;

        let mut end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time {
            let mut pipe = 15u8;
            if self
                .radio
                .available_pipe(&mut pipe)
                .map_err(|e| anyhow!("{e:?}"))?
            {
                let mut incoming_payload = [0u8; 8];
                let len = self
                    .radio
                    .read(&mut incoming_payload, None)
                    .map_err(|e| anyhow!("{e:?}"))?;
                self.counter = incoming_payload[7];
                // print pipe number and payload length and payload
                println!(
                    "Received {len} bytes on pipe {pipe}: {}{} Sent: {}{}",
                    String::from_utf8_lossy(&incoming_payload[0..6]),
                    self.counter,
                    String::from_utf8_lossy(&outgoing_payload[0..6]),
                    outgoing_payload[7],
                );
                // reset timeout
                end_time = Instant::now() + Duration::from_secs(timeout as u64);

                // increment counter
                self.counter += 1;
                outgoing_payload[7] = self.counter;
                // load new ACK payload for next response
                self.radio
                    .write_ack_payload(1, &outgoing_payload)
                    .map_err(|e| anyhow!("{e:?}"))?;
            }
        }

        // It is highly recommended to keep the radio idling in an inactive TX mode
        self.radio.as_tx().map_err(|e| anyhow!("{e:?}"))?;
        // as_tx() will also flush any remaining ACK payloads from the radio's TX FIFO.
        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'R' for receiver role.\n\
        *** Enter 'T' for transmitter role.\n\
        *** Enter 'Q' to quit example.";
        println!("{prompt}");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
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
            self.radio.power_down().map_err(|e| anyhow!("{e:?}"))?;
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

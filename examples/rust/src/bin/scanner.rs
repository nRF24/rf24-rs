//! This is an example of how to use the nRF24L01's builtin
//! Received Power Detection (RPD) to scan for possible interference.
//!
//! This example does not require a counterpart node.
//!
//! The output of the scanner example is supposed to be read vertically (as columns).
//! So, the following
//!
//! ```text
//! 000
//! 111
//! 789
//! ~~~
//! 13-
//! ```
//!
//! should be interpreted as
//!
//! - `1` signal detected on channel `017`
//! - `3` signals detected on channel `018`
//! - no signal (`-`) detected on channel `019`
//!
//! The `~` is just a divider between the vertical header and the signal counts.
//!
//! See documentation at <https://docs.rs/rf24-rs>
#![no_std]

use anyhow::Result;
use core::time::Duration;
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    CrcLength, DataRate, FifoState,
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
    io::{stdin, stdout, Write},
    string::{String, ToString},
    time::Instant,
};

/// The maximum number of supported channels.
const MAX_CHANNELS: u8 = 126;

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
    /// We will be using a 32-bit float value (little-endian) as our payload.
    channel: u8,
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
            channel: 0,
        })
    }

    /// Setup the radio for this example.
    ///
    /// This will initialize and configure the [`App::radio`] object.
    pub fn setup(&mut self, data_rate: DataRate) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(debug_err)?;

        // turn off RX features specific to the nRF24L01 module
        self.radio.set_auto_ack(false).map_err(debug_err)?;
        self.radio.set_dynamic_payloads(false).map_err(debug_err)?;
        self.radio
            .set_crc_length(CrcLength::Disabled)
            .map_err(debug_err)?;
        self.radio.set_data_rate(data_rate).map_err(debug_err)?;

        // use reverse engineering tactics for a better "snapshot"
        self.radio.set_address_length(2).map_err(debug_err)?;
        // The worst possible addresses. These are designed to confuse the radio into thinking
        // the RF signal's preamble is part of the packet/payload.
        let noise_addresses = [
            b"\x55\x55".to_owned(),
            b"\xaa\xaa".to_owned(),
            b"\xa0\xaa".to_owned(),
            b"\x0a\xaa".to_owned(),
            b"\xa5\xaa".to_owned(),
            b"\x5a\xaa".to_owned(),
        ];
        for (pipe, address) in noise_addresses.iter().enumerate() {
            self.radio
                .open_rx_pipe(pipe as u8, address)
                .map_err(debug_err)?;
        }
        Ok(())
    }

    /// The scanner behavior.
    ///
    /// Uses the [`App::radio`] to detect ambient noise.
    pub fn scan(&mut self, timeout: u8) -> Result<()> {
        // print vertical header
        for ch in 0..MAX_CHANNELS {
            print!("{}", ch / 100);
        }
        for ch in 0..MAX_CHANNELS {
            print!("{}", (ch / 10) % 10);
        }
        for ch in 0..MAX_CHANNELS {
            print!("{}", ch % 10);
        }
        for _ in 0..MAX_CHANNELS {
            print!("~");
        }
        println!();

        let mut sweeps = 0u8;
        let mut signals = [0u8; MAX_CHANNELS as usize];
        let end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time {
            self.radio.set_channel(self.channel).map_err(debug_err)?;
            // wait for radio to settle on new channel
            DelayImpl.delay_us(10);

            // scan the current channel
            self.radio.as_rx().map_err(debug_err)?;
            DelayImpl.delay_us(130);
            let found_signal = self.radio.rpd().map_err(debug_err)?;
            self.radio.as_tx().map_err(debug_err)?;
            let found_signal = if self.radio.available().map_err(debug_err)? {
                // discard any packets (noise) saved in RX FIFO
                self.radio.flush_rx().map_err(debug_err)?;
                true
            } else {
                found_signal || self.radio.rpd().map_err(debug_err)?
            };
            if found_signal {
                signals[self.channel as usize] += 1;
            }
            let signal = signals[self.channel as usize];
            if signal == 0 {
                print!("-");
            } else {
                print!("{:X}", signal);
            }

            let mut endl = false;
            self.channel = if self.channel < (MAX_CHANNELS - 1) {
                self.channel + 1
            } else {
                sweeps += 1;
                if sweeps >= 0x0F {
                    endl = true;
                    sweeps = 0;
                    // reset total signal counts for all channels
                    signals = [0u8; MAX_CHANNELS as usize];
                }
                0
            };
            if self.channel == 0 {
                if endl {
                    println!();
                } else {
                    print!("\r");
                }
            }
            // flush stdout to ensure display are updated
            stdout().flush()?;
        }

        // finish printing current cache of signals
        for ch in self.channel..MAX_CHANNELS {
            let signal = signals[ch as usize];
            if signal == 0 {
                print!("-");
            } else {
                print!("{:X}", signal);
            }
        }
        println!();
        Ok(())
    }

    /// print a stream of detected noise for duration of time.
    pub fn noise(&mut self, timeout: u8, channel: Option<u8>) -> Result<()> {
        if let Some(channel) = channel {
            self.radio.set_channel(channel).map_err(debug_err)?;
        }
        self.radio.as_rx().map_err(debug_err)?;
        let end_time = Instant::now() + Duration::from_secs(timeout as u64);
        let mut noise_payload = [0u8; 32];
        while self.radio.is_rx()
            || self.radio.get_fifo_state(false).map_err(debug_err)? != FifoState::Empty
        {
            if Instant::now() > end_time && self.radio.is_rx() {
                self.radio.as_tx().map_err(debug_err)?;
            }
            self.radio
                .read(&mut noise_payload, Some(32))
                .map_err(debug_err)?;
            for byte in noise_payload {
                print!("{byte:02X} ");
            }
        }
        println!();
        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'S' to scan.\n\
        *** Enter 'N' to print ambient noise.\n\
        *** Enter 'Q' to quit example.";
        println!("{prompt}");
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let mut inputs = input.trim().split(' ');
        let role = inputs
            .next()
            .map(|v| v.to_uppercase())
            .unwrap_or("?".to_string());
        if role.starts_with('S') {
            let timeout = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(30);
            self.scan(timeout)?;
            return Ok(true);
        } else if role.starts_with('N') {
            let timeout = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(6);
            let channel = inputs.next().and_then(|i| i.parse::<u8>().ok());
            self.noise(timeout, channel)?;
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
    println!(
        "!!!Make sure the terminal is wide enough for 126 characters on 1 line. \
        If this line is wrapped, then the output will look bad!"
    );
    println!(
        "\nSelect the desired DataRate: (defaults to 1 Mbps)\n\
        1. 1 Mbps\n2. 2 Mbps\n3. 250 Kbps\n"
    );
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let data_rate = match input.trim().chars().next() {
        Some('2') => DataRate::Mbps2,
        Some('3') => DataRate::Kbps250,
        _ => DataRate::Mbps1,
    };
    app.setup(data_rate)?;
    while app.set_role()? {}
    Ok(())
}

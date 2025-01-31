#![no_std]

use core::time::Duration;
use std::{io::Write, string::ToString};

use anyhow::{anyhow, Result};
use rf24::{
    radio::{prelude::*, RF24},
    FifoState, PaLevel, StatusFlags,
};
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    print, println, BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl,
    SpidevDevice as SpiImpl,
};
#[cfg(feature = "linux")]
extern crate std;

/// The length of the stream and the length of each payload within the stream.
const SIZE: usize = 32;

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
    pub fn setup(&mut self, radio_number: u8) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(|e| anyhow!("{e:?}"))?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio
            .set_pa_level(PaLevel::Low)
            .map_err(|e| anyhow!("{e:?}"))?;

        // we'll be using a 32-byte payload lengths
        self.radio
            .set_payload_length(SIZE as u8)
            .map_err(|e| anyhow!("{e:?}"))?;

        let address = [b"1Node", b"2Node"];
        self.radio
            .open_rx_pipe(1, address[radio_number as usize])
            .map_err(|e| anyhow!("{e:?}"))?;
        self.radio
            .open_tx_pipe(address[1 - radio_number as usize])
            .map_err(|e| anyhow!("{e:?}"))?;
        Ok(())
    }

    /// return a list of payloads
    fn make_payloads() -> [[u8; SIZE]; SIZE] {
        // we'll use `size` for the number of payloads in the list and the
        // payloads' length
        let mut stream = [[0u8; SIZE]; SIZE];
        static MAX_LEN: i8 = SIZE as i8 - 1;
        static HALF_LEN: i8 = MAX_LEN / 2;
        for i in 0..SIZE as i8 {
            // prefix payload with a sequential letter to indicate which
            // payloads were lost (if any)
            let buff = &mut stream[i as usize];
            buff[0] = (i as u8) + if i < 26 { 65 } else { 71 };
            let abs_diff = (HALF_LEN - i).abs_diff(0) as i8;
            for j in 0..MAX_LEN {
                let c = j >= (HALF_LEN + abs_diff) || j < (HALF_LEN - abs_diff);
                buff[j as usize + 1] = c as u8 + 48;
            }
        }
        stream
    }

    /// The TX role.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    pub fn tx(&mut self, count: u8) -> Result<()> {
        // create a stream of data
        let stream = Self::make_payloads();
        // declare mutable flags for error checking
        let mut flags = StatusFlags::default();
        // put radio into TX mode
        self.radio.as_tx().map_err(|e| anyhow!("{e:?}"))?;
        for _ in 0..count {
            self.radio.flush_tx().map_err(|e| anyhow!("{e:?}"))?;
            let mut failures = 0u8;
            // start a timer
            let start = std::time::Instant::now();
            for buf in &stream {
                while !self
                    .radio
                    .write(buf, false, true)
                    .map_err(|e| anyhow!("{e:?}"))?
                {
                    // upload to TX FIFO failed because TX FIFO is full.
                    // check for transmission errors
                    self.radio.get_status_flags(&mut flags);
                    if flags.tx_df() {
                        // a transmission failed
                        failures += 1; // increment manual retry count
                        if failures > 99 {
                            // too many failures detected
                            // we need to prevent an infinite loop
                            println!("Make sure other node is listening. Aborting stream");
                            break; // receiver radio seems unresponsive
                        }

                        // rewrite() resets the tx_df flag and reuses top level of TX FIFO
                        self.radio.rewrite().map_err(|e| anyhow!("{e:?}"))?;
                    }
                }
                if failures > 99 {
                    break; // receiver radio seems unresponsive
                }
            }
            // wait for radio to finish transmitting everything in the TX FIFO
            while failures < 99
                && self
                    .radio
                    .get_fifo_state(true)
                    .map_err(|e| anyhow!("{e:?}"))?
                    != FifoState::Empty
            {
                self.radio.get_status_flags(&mut flags);
                if flags.tx_df() {
                    failures += 1;
                    self.radio.rewrite().map_err(|e| anyhow!("{e:?}"))?;
                }
            }
            let end = std::time::Instant::now(); // end timer
            println!(
                "Transmission took {} ms with {} failures detected",
                end.saturating_duration_since(start).as_millis(),
                failures,
            );
        }
        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // put radio into active RX mode
        self.radio.as_rx().map_err(|e| anyhow!("{e:?}"))?;
        let mut count = 0u16;
        let mut end_time =
            std::time::Instant::now() + Duration::from_secs(timeout as u64);
        while std::time::Instant::now() < end_time {
            if self.radio.available().map_err(|e| anyhow!("{e:?}"))? {
                count += 1;
                let mut buf = [0u8; SIZE];
                self.radio
                    .read(&mut buf, None)
                    .map_err(|e| anyhow!("{e:?}"))?;
                // print payload and counter
                println!(
                    "Received: {} - {count}",
                    std::string::String::from_utf8_lossy(&buf)
                );
                // reset timeout
                end_time =
                    std::time::Instant::now() + Duration::from_secs(timeout as u64);
            }
        }

        // It is highly recommended to keep the radio idling in an inactive TX mode
        self.radio.as_tx().map_err(|e| anyhow!("{e:?}"))?;
        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'R' for receiver role.\n\
        *** Enter 'T' for transmitter role.\n\
        *** Enter 'Q' to quit example.";
        println!("{prompt}");
        let mut input = std::string::String::new();
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
                .unwrap_or(1);
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
    let mut input = std::string::String::new();
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

//! Simple example of detecting (and verifying) the IRQ (interrupt) pin on the
//! nRF24L01.
//!
//! This example is meant to be run on 2 separate nRF24L01 transceivers.
//!
//! See documentation at https://docs.rs/rf24-rs
#![no_std]

use anyhow::Result;
use core::time::Duration;
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    FifoState, PaLevel, StatusFlags,
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
    irq_pin: DigitalOutImpl,
    pl_iterator: u8,
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

        let irq_pin = board.default_irq_pin()?;
        Ok(Self {
            board,
            radio,
            irq_pin,
            pl_iterator: 0,
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

        // we'll be using a automatic ACK payloads, so enable the dynamic payload length feature
        self.radio.set_dynamic_payloads(true).map_err(debug_err)?;
        // enable ACK packet payloads
        self.radio.set_ack_payloads(true).map_err(debug_err)?;

        let address = [b"1Node", b"2Node"];
        self.radio
            .open_tx_pipe(address[radio_number as usize])
            .map_err(debug_err)?;
        self.radio
            .open_rx_pipe(1, address[1 - radio_number as usize])
            .map_err(debug_err)?;
        Ok(())
    }

    /// Wait for event to occur.
    ///
    /// Detects change in IRQ pin level.
    fn wait_for_irq(&mut self, timeout: u8) -> Result<bool> {
        let end_time = Instant::now() + Duration::from_secs(timeout as u64);
        let mut event_occurred = false;
        while Instant::now() < end_time && !event_occurred {
            event_occurred = self.irq_pin.get_value()? == 0;
        }
        if !event_occurred {
            println!("\tInterrupt event not detected for {timeout} seconds!");
            return Ok(false);
        }
        Ok(true)
    }

    /// This function is called when IRQ pin is detected active LOW
    fn interrupt_handler(&mut self) -> Result<()> {
        println!("\tIRQ pin went active LOW.");
        // update IRQ status flags
        self.radio.update().map_err(debug_err)?;
        let mut flags = StatusFlags::default();
        self.radio.get_status_flags(&mut flags);
        println!("\t{flags:?}");
        if self.pl_iterator == 0 {
            println!(
                "'data ready' event test {}",
                if flags.rx_dr() { "passed" } else { "failed" }
            );
        } else if self.pl_iterator == 1 {
            println!(
                "'data sent' event test {}",
                if flags.tx_ds() { "passed" } else { "failed" }
            );
        } else if self.pl_iterator == 2 {
            println!(
                "'data fail' event test {}",
                if flags.tx_df() { "passed" } else { "failed" }
            );
        }
        // clear all status flags
        self.radio
            .clear_status_flags(StatusFlags::new())
            .map_err(debug_err)
    }

    /// The TX role.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    /// Transmits 4 times and reports results.
    ///
    /// 1. successfully receive ACK payload first
    /// 2. successfully transmit on second
    /// 3. send a third payload to fill RX node's RX FIFO
    ///    (supposedly making RX node unresponsive)
    /// 4. intentionally fail transmit on the fourth
    pub fn tx(&mut self) -> Result<()> {
        let tx_payloads = [b"Ping ", b"Pong ", b"Radio", b"FAIL!"];
        // put radio into TX mode
        self.radio.as_tx().map_err(debug_err)?;

        // on data ready test
        println!("\nConfiguring IRQ pin to only ignore 'on data sent' event");
        let flags = StatusFlags::new().with_tx_ds(false);
        self.radio.set_status_flags(flags).map_err(debug_err)?;
        println!("    Pinging slave node for an ACK payload...");
        self.pl_iterator = 0;
        self.radio
            .write(tx_payloads[0], false, true)
            .map_err(debug_err)?;
        if self.wait_for_irq(5)? {
            self.interrupt_handler()?;
        }

        // on "data sent" test
        println!("\nConfiguring IRQ pin to only ignore 'on data ready' event");
        let flags = StatusFlags::new().with_rx_dr(false);
        self.radio.set_status_flags(flags).map_err(debug_err)?;
        println!("    Pinging slave node again...");
        self.pl_iterator = 1;
        self.radio
            .write(tx_payloads[1], false, true)
            .map_err(debug_err)?;
        if self.wait_for_irq(5)? {
            self.interrupt_handler()?;
        }

        // trigger slave node to exit by filling the slave node's RX FIFO
        println!("\nSending one extra payload to fill RX FIFO on slave node.");
        println!("Disabling IRQ pin for all events.");
        self.radio
            .set_status_flags(StatusFlags::default())
            .map_err(debug_err)?;
        if self.radio.send(tx_payloads[2], false).map_err(debug_err)? {
            println!("Slave node should not be listening anymore.");
        } else {
            println!("Slave node was unresponsive.");
        }
        self.radio
            .clear_status_flags(StatusFlags::new())
            .map_err(debug_err)?;

        // on "data fail" test
        println!("\nConfiguring IRQ pin to go active for all events.");
        self.radio
            .set_status_flags(StatusFlags::new())
            .map_err(debug_err)?;
        println!("    Sending a ping to inactive slave node...");
        // just in case any previous tests failed, flush the TX FIFO before sending
        self.radio.flush_tx().map_err(debug_err)?;
        self.pl_iterator = 2;
        self.radio
            .write(tx_payloads[3], false, true)
            .map_err(debug_err)?;
        if self.wait_for_irq(5)? {
            self.interrupt_handler()?;
        }

        // flush artifact payload in TX FIFO from last test
        self.radio.flush_tx().map_err(debug_err)?;
        // all 3 ACK payloads received were 4 bytes each, and RX FIFO is full
        // so, fetching 12 bytes from the RX FIFO also flushes RX FIFO
        let mut rx_data = [0u8; 12];
        self.radio.read(&mut rx_data, Some(12)).map_err(debug_err)?;
        println!("\nComplete RX FIFO: {}", String::from_utf8_lossy(&rx_data));

        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    /// Only listen for 3 payload from the master node
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // the "data ready" event will trigger in RX mode
        // the "data sent" or "data fail" events will trigger when we
        // receive with ACK payloads enabled (& loaded in TX FIFO)
        println!("\nDisabling IRQ pin for all events.");
        self.radio
            .set_status_flags(StatusFlags::default())
            .map_err(debug_err)?;
        // setup radio to receive pings, fill TX FIFO with ACK payloads
        let ack_payloads = [b"Yak ", b"Back", b" Ack"];
        for ack in ack_payloads {
            self.radio.write_ack_payload(1, ack).map_err(debug_err)?;
        }

        self.radio.as_rx().map_err(debug_err)?; // start listening & clear irq_dr flag
        let end_time = Instant::now() + Duration::from_secs(timeout as u64); // set end time
        while Instant::now() < end_time
            && self.radio.get_fifo_state(false).map_err(debug_err)? != FifoState::Full
        {
            // wait for RX FIFO to fill up or until timeout is reached
        }
        DelayImpl.delay_ms(500); // wait for last ACK payload to transmit

        // exit TX mode
        self.radio.as_tx().map_err(debug_err)?; // also clears the TX FIFO when ACK payloads are enabled

        // if RX FIFO is not empty (timeout did not occur)
        if self.radio.available().map_err(debug_err)? {
            // all 3 payloads received were 5 bytes each, and RX FIFO is full
            // so, fetching 15 bytes from the RX FIFO also flushes RX FIFO
            let mut rx_data = [0u8; 15];
            self.radio.read(&mut rx_data, Some(15)).map_err(debug_err)?;
            println!("Complete RX FIFO: {}", String::from_utf8_lossy(&rx_data));
        }
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
            self.tx()?;
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

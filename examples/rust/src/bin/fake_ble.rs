//! This example uses the nRF24L01 as a 'fake' BLE Beacon.
//!
//! See rf24-rs documentation at <https://docs.rs/rf24-rs>
//! See rf24ble-rs documentation at <https://docs.rs/rf24ble-rs>
#![no_std]

use anyhow::Result;
use core::time::Duration;
use embedded_hal::delay::DelayNs;

use rf24::{
    radio::{prelude::*, RF24},
    FifoState, PaLevel,
};
use rf24_rs_examples::debug_err;
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    println, BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl, SpidevDevice as SpiImpl,
};
use rf24ble::{
    ble_config,
    services::{prelude::*, BatteryService, TemperatureService, UrlService},
    FakeBle,
};

#[cfg(feature = "linux")]
extern crate std;
#[cfg(feature = "linux")]
use std::{
    io::stdin,
    string::{String, ToString},
    time::Instant,
};

fn prompt(remaining: u8) {
    if remaining > 0 && (remaining % 5 == 0 || remaining < 5) {
        println!("{remaining} advertisements left to go!");
    }
}

/// A struct to drive our example app
struct App {
    /// Any platform-specific functionality is abstracted into this object.
    #[allow(dead_code, reason = "keep board's peripheral objects alive")]
    board: BoardHardware,
    /// Our instantiated RF24 object.
    radio: RF24<SpiImpl, DigitalOutImpl, DelayImpl>,
    /// Our object to wrap BLE behavior around our radio object.
    ble: FakeBle,
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

        let ble = FakeBle::default();

        Ok(Self { board, radio, ble })
    }

    /// Setup the radio for this example.
    ///
    /// This will initialize and configure the [`App::radio`] object.
    pub fn setup(&mut self) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(debug_err)?;

        self.radio.with_config(&ble_config()).map_err(debug_err)?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio.set_pa_level(PaLevel::Low).map_err(debug_err)?;

        Ok(())
    }

    /// Transmits a battery charge level as a BLE beacon.
    pub fn tx_battery(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.as_tx(None).map_err(debug_err)?;

        let mut battery_service = BatteryService::new();
        battery_service.set_data(85); // 85 % remaining charge
        let buf = battery_service.buffer();

        self.ble.set_name("nRF24L01");
        self.ble.show_pa_level = true;

        println!(
            "Number of bytes remaining in advertisement payload: {}",
            self.ble.len_available(buf)
        );

        for remaining in 0..count {
            prompt(count - remaining);
            self.ble.send(&mut self.radio, buf).map_err(debug_err)?;
            self.ble.hop_channel(&mut self.radio).map_err(debug_err)?;
            DelayImpl.delay_ms(500);
        }

        // disable these features for example purposes
        self.ble.set_name("");
        self.ble.show_pa_level = false;
        Ok(())
    }

    /// Transmits a temperature measurement as a BLE beacon.
    pub fn tx_temperature(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.as_tx(None).map_err(debug_err)?;

        let mut temperature_service = TemperatureService::new();
        temperature_service.set_data(45.0); // 45 C degrees
        let buf = temperature_service.buffer();

        self.ble.set_name("nRF24L01");

        println!(
            "Number of bytes remaining in advertisement payload: {}",
            self.ble.len_available(buf)
        );

        for remaining in 0..count {
            prompt(count - remaining);
            self.ble.send(&mut self.radio, buf).map_err(debug_err)?;
            self.ble.hop_channel(&mut self.radio).map_err(debug_err)?;
            DelayImpl.delay_ms(500);
        }

        // disable these features when done (for example purposes)
        self.ble.set_name("");
        Ok(())
    }

    /// Transmits a URL as a BLE beacon.
    pub fn tx_url(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.as_tx(None).map_err(debug_err)?;

        let mut url_service = UrlService::new();
        url_service.set_data("https://www.google.com");
        url_service.set_pa_level(-20);
        let buf = url_service.buffer();

        println!(
            "Number of bytes remaining in advertisement payload: {}",
            self.ble.len_available(buf)
        );

        for remaining in 0..count {
            prompt(count - remaining);
            self.ble.send(&mut self.radio, buf).map_err(debug_err)?;
            self.ble.hop_channel(&mut self.radio).map_err(debug_err)?;
            DelayImpl.delay_ms(500);
        }

        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        // put radio into active RX mode
        self.radio.as_rx().map_err(debug_err)?;

        let end_time = Instant::now() + Duration::from_secs(timeout as u64);
        while Instant::now() < end_time
            || self.radio.get_fifo_state(false).map_err(debug_err)? != FifoState::Empty
        {
            let available = self.radio.available().map_err(debug_err)?;
            if available {
                if let Some(received) = self.ble.read(&mut self.radio).map_err(debug_err)? {
                    println!(
                        "Received payload from MAC address {:02X?}",
                        received.mac_address
                    );
                    if let Some(short_name) = received.short_name {
                        println!("\tDevice name: {}", String::from_utf8_lossy(&short_name));
                    }
                    if let Some(tx_power) = received.tx_power {
                        println!("\tTX power: {tx_power} dBm");
                    }
                    if let Some(battery_charge) = received.battery_charge {
                        println!("\tRemaining battery charge: {} %", battery_charge.data());
                    }
                    if let Some(temperature) = received.temperature {
                        println!("\tTemperature measurement: {} C", temperature.data());
                    }
                    if let Some(url) = received.url {
                        println!("\tURL: {}", url.data());
                    }
                }
                if Instant::now() >= end_time {
                    // It is highly recommended to keep the radio idling in an inactive TX mode
                    self.radio.as_tx(None).map_err(debug_err)?;
                    // continue reading payloads from RX FIFO
                }
            }
        }

        Ok(())
    }

    pub fn set_role(&mut self) -> Result<bool> {
        let prompt = "*** Enter 'R' for receiver role.\n\
        *** Enter 'T' to transmit a temperature measurement.\n\
        *** Enter 'B' to transmit a battery charge level.\n\
        *** Enter 'U' to transmit a URL.\n\
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
                .unwrap_or(50);
            self.tx_temperature(count)?;
            return Ok(true);
        } else if role.starts_with('B') {
            let timeout = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(50);
            self.tx_battery(timeout)?;
            return Ok(true);
        } else if role.starts_with('U') {
            let timeout = inputs
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(50);
            self.tx_url(timeout)?;
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

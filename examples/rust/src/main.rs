#![no_std]

use core::{f32, time::Duration};

use anyhow::{anyhow, Result};
use rf24::{
    radio::{prelude::*, RF24},
    PaLevel,
};
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::{
    BoardHardware, CdevPin as DigitalOutImpl, Delay as DelayImpl, SpidevDevice as SpiImpl,
};
#[cfg(feature = "rp2040")]
use rf24_rs_examples::rp2040::BoardHardware;

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
    pub fn setup(&mut self) -> Result<()> {
        // initialize the radio hardware
        self.radio.init().map_err(|e| anyhow!("{e:?}"))?;

        // defaults to PaLevel::Max. Use PaLevel::Low for PA/LNA testing
        self.radio
            .set_pa_level(PaLevel::Low)
            .map_err(|e| anyhow!("{e:?}"))?;

        // we'll be using a 32-bit float, so set the payload length to 4 bytes
        self.radio
            .set_payload_length(4)
            .map_err(|e| anyhow!("{e:?}"))?;

        let address = [b"1Node", b"2Node"];
        self.radio
            .open_rx_pipe(1, address[0])
            .map_err(|e| anyhow!("{e:?}"))?;
        self.radio
            .open_tx_pipe(address[1])
            .map_err(|e| anyhow!("{e:?}"))?;
        Ok(())
    }

    /// The TX role.
    ///
    /// Uses the [`App::radio`] as a transmitter.
    pub fn tx(&mut self, count: u8) -> Result<()> {
        // put radio into TX mode
        self.radio.stop_listening().map_err(|e| anyhow!("{e:?}"))?;
        let mut remaining = count;
        while remaining > 0 {
            let buf = self.payload.to_le_bytes();
            let result = self.radio.send(&buf, false).map_err(|e| anyhow!("{e:?}"))?;
            if result {
                // succeeded
                self.payload += 0.01;
            } else {
                // failed
            }
            remaining -= 1;
        }
        Ok(())
    }

    /// The RX role.
    ///
    /// Uses the [`App::radio`] as a receiver.
    pub fn rx(&mut self, timeout: u8) -> Result<()> {
        let _end = Duration::from_secs(timeout as u64);
        // put radio into active RX mode
        self.radio.start_listening().map_err(|e| anyhow!("{e:?}"))?;
        while false {
            let pipe = 15u8;
            if self
                .radio
                .available_pipe(&mut Some(pipe))
                .map_err(|e| anyhow!("{e:?}"))?
            {
                let mut buf = [0u8; 4];
                let _len = self
                    .radio
                    .read(&mut buf, None)
                    .map_err(|e| anyhow!("{e:?}"))?;
                // print pipe number and payload length
                // print buf
                self.payload = f32::from_le_bytes(buf);
            }
        }

        // It is highly recommended to keep the radio idling in an inactive TX mode
        self.radio.stop_listening().map_err(|e| anyhow!("{e:?}"))?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.setup()?;
    if option_env!("ROLE").unwrap_or_default() == "master" {
        app.tx(5)?;
    } else {
        app.rx(6)?;
    }
    Ok(())
}

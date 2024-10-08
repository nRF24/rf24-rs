use anyhow::{anyhow, Result};
use rf24_rs::radio::{prelude::*, RF24};
#[cfg(feature = "linux")]
use rf24_rs_examples::linux::BoardHardware;
#[cfg(feature = "rp2040")]
use rf24_rs_examples::rp2040::BoardHardware;

fn main() -> Result<()> {
    // instantiate a hardware peripherals on the board
    let board = BoardHardware::default()?;

    // instantiate radio object using board's hardware
    let mut radio = RF24::new(board.ce_pin, board.spi, board.delay);

    // initialize the radio hardware
    radio.init().map_err(|e| anyhow!("{e:?}"))
}

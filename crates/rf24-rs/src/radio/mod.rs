//! A module to encapsulate all things related to radio operation.
pub mod prelude;

mod rf24;
pub use rf24::{Nrf24Error, RF24};

mod config;
pub use config::RadioConfig;

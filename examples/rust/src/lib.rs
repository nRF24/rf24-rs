//! A helper library meant to abstract platform-specific implementation details
//! from examples that are supposed to be platform-agnostic.
#![no_std]

pub mod hal_impl_trait;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as hal;

#[cfg(all(feature = "rp2040"))]
pub mod rp2040;
#[cfg(all(feature = "rp2040"))]
pub use rp2040 as hal;

#[cfg(all(not(target_os = "none"), not(target_os = "linux"), not(feature = "rp2040")))]
pub mod stubs;
#[cfg(all(not(target_os = "none"), not(target_os = "linux"), not(feature = "rp2040")))]
pub use stubs as hal;

use anyhow::{anyhow, Error};
use core::fmt::Debug;

pub fn debug_err(err: impl Debug) -> Error {
    anyhow!("{err:?}")
}

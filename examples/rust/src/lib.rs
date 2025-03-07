//! A helper library meant to abstract platform-specific implementation details
//! from examples that are supposed to be platform-agnostic.
#![no_std]
#[cfg(feature = "linux")]
pub mod linux;

use anyhow::{anyhow, Error};
use core::fmt::Debug;

pub fn debug_err(err: impl Debug) -> Error {
    anyhow!("{err:?}")
}

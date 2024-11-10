#[cfg(target_os = "linux")]
mod config;
#[cfg(target_os = "linux")]
mod radio;
mod types;

#[macro_use]
extern crate napi_derive;

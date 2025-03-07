#![cfg(not(target_os = "linux"))]

use anyhow::Result;
use embedded_hal::{delay::DelayNs, digital::{InputPin, OutputPin}, spi::SpiDevice};
use crate::hal_impl_trait::HardwareImpl;

extern crate std;
pub use std::{print, println};

pub struct DelayImpl;
impl DelayNs for DelayImpl {
    fn delay_ns(&mut self, _ns: u32) {
        todo!()
    }
}

pub mod digital {
    pub mod error {
        use anyhow::Error;
        use embedded_hal::digital::{Error as DigitalOutError, ErrorKind};

        #[derive(Debug)]
        pub struct DigitalInOutErrorImpl {
            err: Error,
        }

        impl DigitalInOutErrorImpl {
            /// Fetch inner (concrete) [`Error`]
            pub fn inner(&self) -> &Error {
                &self.err
            }
        }

        impl From<Error> for DigitalInOutErrorImpl {
            fn from(err: Error) -> Self {
                Self { err }
            }
        }

        impl core::fmt::Display for DigitalInOutErrorImpl {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.err)
            }
        }

        impl DigitalOutError for DigitalInOutErrorImpl {
            fn kind(&self) -> ErrorKind {
                ErrorKind::Other
            }
        }
    }

    pub mod output {
        use embedded_hal::digital::{ErrorType, OutputPin};

        #[derive(Default)]
        pub struct DigitalOutImpl;

        impl DigitalOutImpl {
            pub fn new() -> Self {
                Self {}
            }
        }

        impl ErrorType for DigitalOutImpl {
            type Error = super::error::DigitalInOutErrorImpl;
        }

        impl OutputPin for DigitalOutImpl {
            fn set_low(&mut self) -> core::result::Result<(), Self::Error> {
                todo!()
            }

            fn set_high(&mut self) -> core::result::Result<(), Self::Error> {
                todo!()
            }
        }
    }

    pub mod input {
        use embedded_hal::digital::{ErrorType, InputPin};
        #[derive(Default)]
        pub struct DigitalInImpl;

        impl DigitalInImpl {
            pub fn new() -> Self {
                Self {}
            }
        }

        impl ErrorType for DigitalInImpl {
            type Error = super::error::DigitalInOutErrorImpl;
        }

        impl InputPin for DigitalInImpl {
            fn is_high(&mut self) -> Result<bool, Self::Error> {
                todo!()
            }

            fn is_low(&mut self) -> Result<bool, Self::Error> {
                todo!()
            }
        }
    }
}

pub use digital::{input::DigitalInImpl, output::DigitalOutImpl};

pub mod spi {
    use embedded_hal::spi::{ErrorType, Operation, SpiDevice};

    pub mod error {
        use anyhow::Error;
        use embedded_hal::spi::{Error as SpiError, ErrorKind};

        #[derive(Debug)]
        pub struct SpiErrorImpl {
            err: Error,
        }

        impl SpiErrorImpl {
            pub fn inner(&self) -> &Error {
                &self.err
            }
        }

        impl SpiError for SpiErrorImpl {
            fn kind(&self) -> ErrorKind {
                ErrorKind::Other
            }
        }

        impl core::fmt::Display for SpiErrorImpl {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.err)
            }
        }
    }

    #[derive(Debug)]
    pub struct SpiImpl;

    impl ErrorType for SpiImpl {
        type Error = error::SpiErrorImpl;
    }

    impl SpiDevice for SpiImpl {
        fn transaction(
            &mut self,
            _operations: &mut [Operation<'_, u8>],
        ) -> Result<(), Self::Error> {
            todo!()
        }
    }
}

pub use spi::SpiImpl;

#[derive(Debug)]
pub struct BoardHardware;
impl HardwareImpl for BoardHardware {
    fn new() -> Result<Self> {
        Ok(Self {})
    }

    fn default_ce_pin(&mut self) -> Result<impl OutputPin> {
        Ok(DigitalOutImpl)
    }

    fn default_spi_device() -> Result<impl SpiDevice> {
        Ok(SpiImpl)
    }

    fn default_irq_pin(&mut self) -> Result<impl InputPin> {
        Ok(DigitalInImpl)
    }
}

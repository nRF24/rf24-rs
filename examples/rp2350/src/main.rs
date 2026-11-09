#![no_std]
#![no_main]

use core::fmt::Write as _;
use core::panic::PanicInfo;

use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::MODE_0};
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::{clocks::Clock, fugit::RateExtU32, gpio::FunctionSpi};
use heapless::String;
use rf24::{
    radio::{prelude::*, RadioConfig, RF24},
    CrcLength, DataRate, PaLevel,
};
use rp235x_hal as hal;
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Set to true on one board, false on the other.
const INITIATOR: bool = true;

const PING: &[u8] = b"PING";
const PONG: &[u8] = b"PONG";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hal::arch::wfi();
    }
}

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[hal::entry]
fn main() -> ! {
    // ============================================================
    // 1. BOARD INITIALIZATION
    //    Clocks, GPIO, peripherals
    // ============================================================

    const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI0 pins
    let miso = pins.gpio16.into_function::<FunctionSpi>();
    let sck = pins.gpio18.into_function::<FunctionSpi>();
    let mosi = pins.gpio19.into_function::<FunctionSpi>();

    let spi_bus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (mosi, miso, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1.MHz(),
        MODE_0,
    );

    // nRF24 control pins
    let csn = pins.gpio21.into_function::<hal::gpio::FunctionSioOutput>();
    let mut ce = pins.gpio20.into_function::<hal::gpio::FunctionSioOutput>();

    ce.set_low().unwrap();

    // Timer provides the delays required by SPI and RF24.
    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let mut spi_delay = timer;

    // Wrap the SPI bus and CSN into an embedded-hal SpiDevice.
    let spi = ExclusiveDevice::new(spi_bus, csn, &mut spi_delay)
        .ok()
        .unwrap();

    // ============================================================
    // 2. USB CDC — DIAGNOSTICS
    //    USB setup, serial output, polling
    // ============================================================

    // USB CDC uses the RP2350's native USB controller.
    // It is independent of SPI and the nRF24 radio.

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    // Shared USB VID/PID for CDC-ACM devices (Objective Development).
    // See: https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("rf24-rs RP2350 Ping-Pong")
        .device_class(USB_CLASS_CDC)
        .build();

    // ============================================================
    // 3. NRF24L01+ — RADIO
    //    SPI, CE, CSN, RF24 configuration
    // ============================================================

    // Construct the radio driver.
    let mut radio = RF24::new(ce, spi, timer);

    let mut pacing = timer;

    const RADIO_CHANNEL: u8 = 50;
    const RADIO_ADDRESS: [u8; 5] = [0xE7; 5];

    let config = RadioConfig::default()
        .with_channel(RADIO_CHANNEL)
        .with_data_rate(DataRate::Mbps1)
        .with_crc_length(CrcLength::Bit8)
        .with_pa_level(PaLevel::Min)
        .with_tx_address(&RADIO_ADDRESS)
        .with_rx_address(0, &RADIO_ADDRESS)
        .close_rx_pipe(1)
        .with_dynamic_payloads(true)
        .with_auto_ack(1);

    radio.init().unwrap();
    radio.with_config(&config).unwrap();

    // Both roles start listening.
    // The initiator will switch to TX when sending PING.
    radio.as_rx().unwrap();

    // ============================================================
    // 4. APPLICATION — PING/PONG
    //    Initiator and responder behavior
    // ============================================================

    let mut tx_count = 0u32;
    let mut rx_count = 0u32;

    // Initiator state.
    let mut waiting_for_pong = false;
    let mut wait_ms = 0u32;
    let mut interval_ms = 1000u32;

    loop {
        // USB CDC needs regular polling, independently of RF24.
        usb_dev.poll(&mut [&mut serial]);

        // Each iteration may generate one diagnostic message.
        let mut message: String<96> = String::new();

        if INITIATOR {
            if waiting_for_pong {
                if radio.available().unwrap() {
                    let mut buf = [0u8; 32];
                    let len = radio.read(&mut buf, None).unwrap();

                    if &buf[..len as usize] == PONG {
                        rx_count = rx_count.wrapping_add(1);
                        let _ = write!(message, "[RF24] RX PONG #{}\r\n", rx_count);
                    } else {
                        let _ = write!(message, "[RF24] Unexpected payload\r\n");
                    }

                    waiting_for_pong = false;
                    interval_ms = 0;
                } else {
                    wait_ms += 1;

                    if wait_ms >= 500 {
                        let _ = write!(message, "[RF24] PONG timeout\r\n");

                        waiting_for_pong = false;
                        interval_ms = 0;
                    }
                }
            } else {
                interval_ms += 1;

                if interval_ms >= 1000 {
                    radio.as_tx(None).unwrap();

                    let sent = radio.send(PING, false).unwrap();

                    radio.as_rx().unwrap();

                    tx_count = tx_count.wrapping_add(1);

                    if sent {
                        let _ = write!(message, "[RF24] TX PING #{}: OK\r\n", tx_count);

                        waiting_for_pong = true;
                        wait_ms = 0;
                    } else {
                        radio.flush_tx().unwrap();

                        let _ = write!(message, "[RF24] TX PING #{}: FAILED\r\n", tx_count);

                        interval_ms = 0;
                    }
                }
            }
        } else {
            // Responder continuously listens for PING.
            if radio.available().unwrap() {
                let mut buf = [0u8; 32];
                let len = radio.read(&mut buf, None).unwrap();

                if &buf[..len as usize] == PING {
                    rx_count = rx_count.wrapping_add(1);

                    // Let the initiator return to RX mode.
                    pacing.delay_ms(5);

                    radio.as_tx(None).unwrap();

                    let sent = radio.send(PONG, false).unwrap();

                    if !sent {
                        radio.flush_tx().unwrap();
                    }

                    radio.as_rx().unwrap();

                    tx_count = tx_count.wrapping_add(1);

                    let _ = write!(
                        message,
                        "[RF24] RX PING #{} -> TX PONG #{}: {}\r\n",
                        rx_count,
                        tx_count,
                        if sent { "OK" } else { "FAILED" }
                    );
                }
            }
        }

        // USB CDC — display radio events on the host terminal.
        //
        // Logging is best-effort: if USB is unavailable or its
        // buffer is full, radio communication continues normally.
        if usb_dev.state() == UsbDeviceState::Configured && !message.is_empty() {
            let _ = serial.write(message.as_bytes());
        }

        // Give the USB stack another opportunity to progress.
        usb_dev.poll(&mut [&mut serial]);

        pacing.delay_ms(1);
    }
}

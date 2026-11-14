#![no_std]
#![no_main]

use esp_backtrace as _;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Duration, Rate},
};
use esp_println::println;
use rf24::{
    radio::{prelude::*, RadioConfig, RF24},
    CrcLength, DataRate, PaLevel,
};

// Set to true on one board, false on the other.
const INITIATOR: bool = false;

// Each node has its own RX address.
const ADDRESSES: [[u8; 5]; 2] = [*b"Node1", *b"Node2"];

const PING: &[u8] = b"PING";
const PONG: &[u8] = b"PONG";

const RESPONSE_DELAY_MS: u64 = 5;
const PONG_TIMEOUT_MS: u32 = 500;
const PING_INTERVAL_MS: u32 = 1000;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // ============================================================
    // 1. BOARD INITIALIZATION
    //    Clocks, GPIO, peripherals
    // ============================================================

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // ============================================================
    // 2. CONSOLE — DIAGNOSTICS
    //    Output using esp-println
    // ============================================================

    println!("RF24 ping-pong - ESP32-C3");
    println!("SPI2: 1 MHz, Mode 0");
    println!("SCK=GPIO8 MOSI=GPIO10 MISO=GPIO21");
    println!("CSN=GPIO5 CE=GPIO4");
    println!("RF Channel: {}", RADIO_CHANNEL);
    println!(
        "Role: {}",
        if INITIATOR { "Initiator" } else { "Responder" }
    );

    // ============================================================
    // 3. NRF24L01+ — RADIO
    //    SPI, CE, CSN, RF24 configuration
    // ============================================================

    let spi_bus = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(1))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO8)
    .with_mosi(peripherals.GPIO10)
    .with_miso(peripherals.GPIO21);

    let csn = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    let ce = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    let spi = ExclusiveDevice::new(spi_bus, csn, Delay::new()).unwrap();

    let mut radio = RF24::new(ce, spi, Delay::new());

    /// Enable hardware auto-ACK on RX pipes 0 and 1.
    ///
    /// Pipe 0 is used internally to receive ACKs during transmission.
    /// Pipe 1 receives application payloads (PING/PONG).
    /// Bits 0 and 1 correspond to RX pipes 0 and 1, respectively.
    const AUTO_ACK_PIPES: u8 = (1 << 0) | (1 << 1);
    const RADIO_CHANNEL: u8 = 50;

    let config = RadioConfig::default()
        .with_channel(RADIO_CHANNEL)
        .with_data_rate(DataRate::Mbps1)
        .with_crc_length(CrcLength::Bit8)
        .with_pa_level(PaLevel::Min)
        .with_tx_address(&ADDRESSES[INITIATOR as usize])
        .with_rx_address(1, &ADDRESSES[!INITIATOR as usize])
        .with_dynamic_payloads(true)
        .with_auto_ack(AUTO_ACK_PIPES);

    let delay = Delay::new();

    if let Err(error) = radio.init().and_then(|_| radio.with_config(&config)) {
        println!("[FAIL] Radio initialization: {:?}", error);
        loop {
            delay.delay(Duration::from_secs(2));
        }
    }

    if let Err(error) = radio.as_rx() {
        println!("[FAIL] Enter RX: {:?}", error);
        loop {
            delay.delay(Duration::from_secs(2));
        }
    }

    println!("[PASS] Radio initialized");
    println!("[RX] Listening on channel {}", RADIO_CHANNEL);

    // ============================================================
    // 4. APPLICATION — PING/PONG
    //    Initiator and responder behavior
    // ============================================================

    let mut tx_count = 0u32;
    let mut rx_count = 0u32;

    let mut waiting_for_pong = false;
    let mut wait_ms = 0u32;
    // Start at the threshold to send the first PING immediately.
    let mut interval_ms = PING_INTERVAL_MS;

    let mut buffer = [0u8; 32];

    loop {
        if INITIATOR {
            if waiting_for_pong {
                if radio.available().unwrap() {
                    let length = radio.read(&mut buffer, None).unwrap();
                    let payload = &buffer[..usize::from(length)];

                    if payload == PONG {
                        rx_count = rx_count.wrapping_add(1);
                        println!("[RF24] RX PONG #{}", rx_count);
                    } else {
                        println!("[RF24] Unexpected payload: {:02X?}", payload);
                    }

                    waiting_for_pong = false;
                    interval_ms = 0;
                } else {
                    wait_ms += 1;

                    if wait_ms >= PONG_TIMEOUT_MS {
                        println!("[RF24] PONG timeout");

                        waiting_for_pong = false;
                        interval_ms = 0;
                    }
                }
            } else {
                interval_ms += 1;

                if interval_ms >= PING_INTERVAL_MS {
                    radio.as_tx(None).unwrap();

                    let sent = radio.send(PING, false).unwrap();

                    radio.as_rx().unwrap();

                    tx_count = tx_count.wrapping_add(1);

                    if sent {
                        println!("[RF24] TX PING #{}: OK", tx_count);

                        waiting_for_pong = true;
                        wait_ms = 0;
                    } else {
                        radio.flush_tx().unwrap();
                        println!("[RF24] TX PING #{}: FAILED", tx_count);

                        interval_ms = 0;
                    }
                }
            }
        } else {
            // Responder continuously listens for PING.
            if radio.available().unwrap() {
                let length = radio.read(&mut buffer, None).unwrap();
                let payload = &buffer[..usize::from(length)];

                if payload == PING {
                    rx_count = rx_count.wrapping_add(1);
                    println!("[RF24] RX PING #{}", rx_count);

                    // Give initiator time to return to RX mode.
                    delay.delay(Duration::from_millis(RESPONSE_DELAY_MS));

                    radio.as_tx(None).unwrap();

                    let sent = radio.send(PONG, false).unwrap();

                    if !sent {
                        radio.flush_tx().unwrap();
                    }

                    radio.as_rx().unwrap();

                    tx_count = tx_count.wrapping_add(1);

                    println!(
                        "[RF24] TX PONG #{}: {}",
                        tx_count,
                        if sent { "OK" } else { "FAILED" }
                    );
                } else {
                    println!("[RF24] Unexpected payload: {:02X?}", payload);
                }
            }
        }

        delay.delay(Duration::from_millis(1));
    }
}

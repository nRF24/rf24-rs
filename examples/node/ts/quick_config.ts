/**
 * This example demonstrates how to quickly and easily
 * change the radio's configuration.
 *
 * This example requires no counterpart as
 * it does not actually transmit nor receive anything.
 *
 * See documentation at https://nRF24.github.io/rf24-rs
 */
import { RF24, RadioConfig, CrcLength } from "@rf24/rf24";

export class App {
  radio: RF24;

  /**
   * @param radioNumber The number (0 or 1) that identifies the which radio is used in
   * this example that requires 2 radios.
   */
  constructor() {
    // The radio's CE Pin uses a GPIO number.
    const cePin = 22; // for GPIO22

    // The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
    // On Linux, consider the device path `/dev/spidev<a>.<b>`:
    //   - `<a>` is the SPI bus number (defaults to `0`)
    //   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
    const csnPin = 0; // aka CE0 for SPI bus 0 (/dev/spidev0.0)

    // create a radio object for the specified hardware config:
    this.radio = new RF24(cePin, csnPin);

    // initialize the nRF24L01 on the spi bus
    this.radio.begin();
  }

  /**
   * Configure the radio for 2 different scenarios and
   * print the configuration details for each.
   */
  run() {
    // Config context 1
    const bleContext = new RadioConfig(); // library defaults
    bleContext.channel = 2; // BLE specs hop/rotate amongst channels 2, 26, and 80
    bleContext.crcLength = CrcLength.Disabled;
    bleContext.autoAck = 0;
    bleContext.addressLength = 4;
    const bleAddr = Buffer.from([0x71, 0x91, 0x7d, 0x6b]);
    bleContext.setRxAddress(1, bleAddr);
    bleContext.txAddress = bleAddr;

    // Config context 2
    const normalContext = new RadioConfig(); // library defaults
    normalContext.setRxAddress(1, Buffer.from("1Node"));
    normalContext.txAddress = Buffer.from("2Node");

    this.radio.withConfig(bleContext);
    console.log("Settings for BLE context\n------------------------");
    this.radio.printDetails();

    this.radio.withConfig(normalContext);
    console.log("\nSettings for normal context\n---------------------------");
    this.radio.printDetails();
    console.log();
  }
}

export async function main() {
  console.log(module.filename);
  const app = new App();
  app.run();
}

main();

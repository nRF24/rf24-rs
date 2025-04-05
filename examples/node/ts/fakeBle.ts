/**
 * This example uses the nRF24L01 as a 'fake' BLE Beacon
 */
import * as readline from "readline/promises";
import * as timer from "timers/promises";
import {
  bleConfig,
  FakeBle,
  FifoState,
  RF24,
  PaLevel,
  BatteryService,
  TemperatureService,
  UrlService,
} from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

function _prompt(remaining: number) {
  if (remaining && (remaining % 5 == 0 || remaining < 5)) {
    console.log(remaining, "advertisements left to go!");
  }
}

export class App {
  radio: RF24;
  ble: FakeBle;

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
    // configure the radio for BLE compatibility
    this.radio.withConfig(bleConfig());

    this.ble = new FakeBle(this.radio);

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    this.radio.paLevel = PaLevel.Low; // PaLevel.Max is default
  }

  /**
   * Transmits a battery charge level as a BLE beacon.
   * @param count The number of payloads to send
   */
  async txBattery(count?: number) {
    this.radio.asTx();

    const batteryService = new BatteryService();
    batteryService.data = 85; // 85  % remaining charge level
    const buffer = batteryService.buffer;

    this.ble.name = "nRF24L01";
    this.ble.showPaLevel = true;

    console.log(
      "Number of bytes remaining in advertisement payload:",
      this.ble.lenAvailable(buffer),
    );

    for (let i = count || 50; i > 0; --i) {
      _prompt(i);
      this.ble.send(buffer);
      this.ble.hopChannel();
      await timer.setTimeout(500);
    }

    // disable these features when done (for example purposes)
    this.ble.name = "";
    this.ble.showPaLevel = false;
  }

  /**
   * Transmits a temperature measurement as a BLE beacon.
   * @param count The number of payloads to send
   */
  async txTemperature(count?: number) {
    this.radio.asTx();

    const temperatureService = new TemperatureService();
    temperatureService.data = 45.5; // 45.5 degrees Celsius
    const buffer = temperatureService.buffer;

    this.ble.name = "nRF24L01";

    console.log(
      "Number of bytes remaining in advertisement payload:",
      this.ble.lenAvailable(buffer),
    );

    for (let i = count || 50; i > 0; --i) {
      _prompt(i);
      this.ble.send(buffer);
      this.ble.hopChannel();
      await timer.setTimeout(500);
    }

    // disable these features when done (for example purposes)
    this.ble.name = "";
  }

  /**
   * Transmits a URL as a BLE beacon.
   * @param count The number of payloads to send
   */
  async txUrl(count?: number) {
    this.radio.asTx();

    const urlService = new UrlService();
    urlService.data = "https://www.google.com";
    urlService.paLevel = -20;
    const buffer = urlService.buffer;

    console.log(
      "Number of bytes remaining in advertisement payload:",
      this.ble.lenAvailable(buffer),
    );

    for (let i = count || 50; i > 0; --i) {
      _prompt(i);
      this.ble.send(buffer);
      this.ble.hopChannel();
      await timer.setTimeout(500);
    }
  }

  /**
   * Polls the radio and prints the received value.
   * @param duration The timeout duration (in seconds) to listen after receiving a payload.
   */
  rx(duration?: number) {
    this.radio.asRx(); // put radio into RX mode

    const timeout = Date.now() + (duration || 6) * 1000;
    while (
      this.radio.getFifoState(false) != FifoState.Empty ||
      Date.now() < timeout
    ) {
      if (this.radio.available()) {
        // fetch 1 payload from RX FIFO
        const received = this.ble.read();
        if (received) {
          const mac = Array<string>();
          received.macAddress.forEach((element) => {
            mac.push(element.toString(16).toUpperCase().padStart(2, "0"));
          });
          console.log("Received payload from MAC address", mac.join(":"));
          if (received.shortName) {
            console.log("\tDevice name:", received.shortName);
          }
          if (received.txPower) {
            console.log("\tTX power:", received.txPower, "dBm");
          }
          if (received.batteryCharge) {
            console.log(
              `\tRemaining battery charge: ${received.batteryCharge.data}%`,
            );
          }
          if (received.temperature) {
            console.log(
              `\tTemperature measurement: ${received.temperature.data} C`,
            );
          }
          if (received.url) {
            console.log("\tURL:", received.url.data);
          }
        }
      }
      if (Date.now() >= timeout) {
        // recommended behavior is to keep in TX mode while idle
        this.radio.asTx(); // exit RX mode
        // continue to read remaining payloads from RX FIFO
      }
    }
    this.radio.asTx();
  }

  /**
   * This function prompts the user and performs the specified role for the radio.
   */
  async setRole() {
    const prompt =
      "*** Enter 'R' for receiver role.\n" +
      "*** Enter 'T' to transmit a temperature measurement.\n" +
      "*** Enter 'B' to transmit a battery charge level.\n" +
      "*** Enter 'U' to transmit a URL.\n" +
      "*** Enter 'Q' to quit example.\n";
    const input = (await io.question(prompt)).split(" ");
    const role = input.shift() || "?";
    const params = Array<number>();
    for (let i = 0; i < input.length; ++i) {
      params.push(Number(input[i]));
    }
    switch (role.charAt(0).toLowerCase()) {
      case "t":
        await this.txTemperature(...params);
        return true;
      case "b":
        await this.txBattery(...params);
        return true;
      case "u":
        await this.txUrl(...params);
        return true;
      case "r":
        this.rx(...params);
        return true;
      default:
        console.log(`'${role.charAt(0)}' is an unrecognized input`);
        return true;
      case "q":
        this.radio.powerDown();
        return false;
    }
  }
}

export async function main() {
  console.log(module.filename);

  const app = new App();
  while (await app.setRole());
  io.close();
}

main();

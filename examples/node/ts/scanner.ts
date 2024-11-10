import * as readline from "readline/promises";
import * as fs from "fs";
import * as timer from "timers/promises";
import { RF24, CrcLength, DataRate, FifoState } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

const CHANNELS = 126;

export class App {
  radio: RF24;

  constructor(dataRate: DataRate) {
    // The radio's CE Pin uses a GPIO number.
    // On Linux, consider the device path `/dev/gpiochip<N>`:
    //   - `<N>` is the gpio chip's identifying number.
    //     Using RPi4 (or earlier), this number is `0` (the default).
    //     Using the RPi5, this number is actually `4`.
    // The radio's CE pin must connected to a pin exposed on the specified chip.
    const cePin = 22; // for GPIO22
    // try detecting RPi5 first; fall back to default
    const gpioChip = fs.existsSync("/dev/gpiochip4") ? 4 : 0;

    // The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
    // On Linux, consider the device path `/dev/spidev<a>.<b>`:
    //   - `<a>` is the SPI bus number (defaults to `0`)
    //   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
    const csnPin = 0; // aka CE0 for SPI bus 0 (/dev/spidev0.0)

    // create a radio object for the specified hardware config:
    this.radio = new RF24(cePin, csnPin, {
      devGpioChip: gpioChip,
    });

    // initialize the nRF24L01 on the spi bus
    this.radio.begin();

    // This is the worst possible configuration.
    // The intention here is to pick up as much noise as possible.
    this.radio.addressLength = 2;

    // For this example, we will use the worst possible addresses
    const address = [
      Buffer.from([0x55, 0x55]),
      Buffer.from([0xaa, 0xaa]),
      Buffer.from([0xa0, 0xaa]),
      Buffer.from([0x0a, 0xaa]),
      Buffer.from([0xa5, 0xaa]),
      Buffer.from([0x5a, 0xaa]),
    ];
    for (let pipe = 0; pipe < address.length; pipe++) {
      this.radio.openRxPipe(pipe, address[pipe]);
    }

    this.radio.dataRate = dataRate;
    // turn off auto-ack related features
    this.radio.setAutoAck(false);
    this.radio.dynamicPayloads = false;
    this.radio.crcLength = CrcLength.Disabled;
  }

  /**
   * Prints the vertical header for all the channels
   */
  printHeader() {
    let hundreds = "";
    let tens = "";
    let ones = "";
    let divider = "";
    for (let i = 0; i < CHANNELS; i++) {
      hundreds += Math.floor(i / 100).toString();
      tens += (Math.floor(i / 10) % 10).toString();
      ones += (i % 10).toString();
      divider += "~";
    }
    console.log(hundreds);
    console.log(tens);
    console.log(ones);
    console.log(divider);
  }

  /**
   * The scanner behavior.
   */
  async scan(duration?: number) {
    this.printHeader();
    const caches = [];
    for (let i = 0; i < CHANNELS; i++) {
      caches.push(0);
    }
    let sweeps = 0;
    let channel = 0;

    const timeout = Date.now() + (duration || 30) * 1000;
    while (Date.now() < timeout) {
      this.radio.channel = channel;
      this.radio.asRx();
      await timer.setTimeout(0.13); // needs to be at least 130 microseconds
      const rpd = this.radio.rpd;
      this.radio.asTx();
      const foundSignal = this.radio.available();

      caches[channel] += Number(foundSignal || rpd || this.radio.rpd);

      if (foundSignal) {
        this.radio.flushRx(); // discard any packets (noise) saved in RX FIFO
      }
      const total = caches[channel];
      process.stdout.write(total > 0 ? total.toString(16) : "-");

      channel += 1;
      let endl = false;
      if (channel >= CHANNELS) {
        channel = 0;
        sweeps += 1;
      }
      if (sweeps >= 15) {
        endl = true;
        sweeps = 0;
        // reset total signal counts for all channels
        for (let i = 0; i < CHANNELS; i++) {
          caches[i] = 0;
        }
      }
      if (channel == 0) {
        process.stdout.write(endl ? "\n" : "\r");
      }
    }

    // finish printing current cache of signals
    for (let i = channel; i < CHANNELS; i++) {
      const total = caches[i];
      process.stdout.write(total > 0 ? total.toString(16) : "-");
    }
  }

  /**
   * Sniff ambient noise and print it out as hexadecimal string.
   */
  noise(duration?: number) {
    const timeout = Date.now() + (duration || 10) * 1000;
    this.radio.asRx();
    while (
      this.radio.isRx ||
      this.radio.getFifoState(false) != FifoState.Empty
    ) {
      const payload = this.radio.read();
      const hexArray = [];
      for (let i = 0; i < payload.length; i++) {
        hexArray.push(payload[i].toString(16).padStart(2, "0"));
      }
      console.log(hexArray.join(" "));
      if (Date.now() > timeout && this.radio.isRx) {
        this.radio.asTx();
      }
    }
  }

  /**
   * This function prompts the user and performs the specified role for the radio.
   */
  async setRole() {
    const prompt =
      "*** Enter 'S' to scan\n" +
      "*** Enter 'N' to print noise\n" +
      "*** Enter 'Q' to quit\n";
    const input = (await io.question(prompt)).split(" ");
    const role = input.shift() || "?";
    const params = Array<number>();
    for (let i = 0; i < input.length; ++i) {
      params.push(Number(input[i]));
    }
    switch (role.charAt(0).toLowerCase()) {
      case "s":
        await this.scan(...params);
        return true;
      case "n":
        this.noise(...params);
        return true;
      default:
        console.log(`'${input[0].charAt(0)}' is an unrecognized input`);
        return true;
      case "q":
        this.radio.powerDown();
        return false;
    }
  }
}

export async function main() {
  console.log(module.filename);
  console.log(
    "!!!Make sure the terminal is wide enough for 126 characters on 1 line." +
      " If this line is wrapped, then the output will look bad!",
  );

  // Ask user for desired data rate (default to 1 Mbps)
  const dRatePrompt =
    "Select the desired DataRate: (defaults to 1 Mbps)\n" +
    "1. 1 Mbps\n2. 2 Mbps\n3. 250 Kbps\n";
  const answer = parseInt(await io.question(dRatePrompt)) || 0;
  let dataRate = DataRate.Mbps1;
  if (answer == 2) {
    dataRate = DataRate.Mbps2;
    console.log(`Data Rate is 2 Mbps`);
  } else if (answer == 3) {
    dataRate = DataRate.Kbps250;
    console.log(`Data Rate is 250 Kbps`);
  } else {
    console.log(`Data Rate is 1 Mbps`);
  }
  const app = new App(dataRate);
  while (await app.setRole());
  io.close();
}

main();

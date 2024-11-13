import * as readline from "readline/promises";
import * as fs from "fs";
import * as timer from "timers/promises";
import * as tui from "terminal-kit";
import { RF24, CrcLength, DataRate } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

const CACHE_MAX = 6;

/**
 * A class to encapsulate a single progress bar for each channel.
 */
export class ProgressBar {
  x: number;
  y: number;
  isColOdd: boolean;
  label: string;
  total: number;
  history: Array<boolean>;
  sum: number;
  width: number;

  constructor(x: number, y: number, label: string, isColOdd: boolean) {
    this.x = x;
    this.y = y;
    this.label = label;
    this.isColOdd = isColOdd;
    this.total = 0;
    this.history = [];
    for (let i = 0; i < CACHE_MAX; ++i) {
      this.history.push(false);
    }
    this.sum = 0;
    this.width = Math.floor(tui.terminal.width / 6) - (label.length + 4);
  }

  /**
   * Update the progress bar's values.
   */
  update(foundSignal: boolean) {
    const oldSum = this.sum;
    this.sum = 0;
    this.history.shift();
    this.history.push(foundSignal);
    this.history.forEach((val) => {
      this.sum += Number(val);
    });
    this.total += Number(foundSignal);
    if (this.sum != oldSum) {
      this.draw();
    }
  }

  /**
   * Draw the progress bar.
   */
  draw() {
    let filled = "";
    const filledWidth = Math.ceil(this.width * (this.sum / CACHE_MAX));
    for (let i = 0; i < filledWidth; ++i) {
      filled += "=";
    }
    let bg = "";
    const bgWidth = this.width - filledWidth;
    for (let i = 0; i < bgWidth; ++i) {
      bg += "-";
    }
    const total =
      this.total == 0
        ? "-"
        : Math.min(this.total, 0xf).toString(16).toUpperCase();
    tui.terminal.moveTo(this.x, this.y);
    if (this.isColOdd) {
      // draw yellow bar
      tui.terminal
        .yellow(`${this.label} `)
        .magenta(filled)
        .yellow(`${bg} ${total} `);
    } else {
      //draw white bar
      tui.terminal
        .white(`${this.label} `)
        .magenta(filled)
        .white(`${bg} ${total} `);
    }
  }
}

const CHANNELS = 126;

export class App {
  radio: RF24;
  progressBars: Array<ProgressBar>;

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

    this.progressBars = Array(CHANNELS);
    const bar_w = Math.floor(tui.terminal.width / 6);
    for (let i = 0; i < 21; ++i) {
      // 21 rows
      for (let j = i; j < i + 21 * 6; j += 21) {
        // 6 columns
        const isColOdd = Math.floor(j / 21) % 2 > 0;
        const label = (2400 + j).toString();
        const y = i + 4;
        const x = bar_w * Math.floor(j / 21) + 1;
        this.progressBars[j] = new ProgressBar(x, y, label, isColOdd);
      }
    }
  }

  /**
   * The scanner behavior.
   */
  async run(duration: number, dataRate: string) {
    let sweeps = 0;
    let channel = 0;
    tui.terminal.clear();
    this.progressBars.forEach((bar) => {
      bar.draw();
    });
    tui.terminal.moveTo(1, 1, "Channels are labeled in Hz.");
    tui.terminal.moveTo(
      1,
      2,
      "Signal counts are clamped to a single hexadecimal digit.",
    );

    const timeout = Date.now() + (duration || 30) * 1000;
    let prevSec = 0;
    while (Date.now() < timeout) {
      await this.scan(channel);

      channel += 1;
      if (channel >= CHANNELS) {
        channel = 0;
        sweeps += 1;
      }
      const currSec = Math.floor(Date.now() / 1000);
      if (currSec != prevSec) {
        const remaining = (Math.floor(timeout / 1000) - currSec)
          .toString()
          .padStart(3);
        tui.terminal.moveTo(
          1,
          3,
          `Scanning for ${remaining} seconds at ${dataRate}.`,
        );
        prevSec = currSec;
      }
    }

    tui.terminal.clear();
    let noisyChannels = 0;
    const sweepsWidth = sweeps.toString().length;
    for (let i = 0; i < CHANNELS; ++i) {
      const total = this.progressBars[i].total;
      const percentage = ((total / sweeps) * 100).toPrecision(3);
      const paddedTotal = total.toString().padStart(sweepsWidth);
      if (total > 0) {
        console.log(
          `    ${i.toString().padStart(3)}: ${paddedTotal}`,
          `/ ${sweeps} (${percentage} %)`,
        );
        noisyChannels += 1;
      }
    }
    console.log(
      `${noisyChannels} channels detected signals out of`,
      `${sweeps} passes on the entire spectrum`,
    );
  }

  /**
   * scan a specified channel
   */
  async scan(channel: number) {
    this.radio.channel = channel;
    this.radio.asRx();
    await timer.setTimeout(0.13); // needs to be at least 130 microseconds
    const rpd = this.radio.rpd;
    this.radio.asTx();
    const foundSignal = this.radio.available() || rpd || this.radio.rpd;

    if (foundSignal) {
      this.radio.flushRx(); // discard any packets (noise) saved in RX FIFO
    }
    this.progressBars[channel].update(foundSignal);
  }
}

export async function main() {
  console.log(module.filename);

  // Ask user for desired data rate (default to 1 Mbps)
  const dRatePrompt =
    "Select the desired DataRate: (defaults to 1 Mbps)\n" +
    "1. 1 Mbps\n2. 2 Mbps\n3. 250 Kbps\n";
  const answer = parseInt(await io.question(dRatePrompt)) || 0;
  let dataRate = DataRate.Mbps1;
  let dataRateString = "1 Mbps";
  if (answer == 2) {
    dataRate = DataRate.Mbps2;
    dataRateString = "2 Mbps";
  } else if (answer == 3) {
    dataRate = DataRate.Kbps250;
    dataRateString = "250 Kbps";
  }
  const app = new App(dataRate);
  let duration = NaN;
  while (Number.isNaN(duration)) {
    duration = parseInt(
      await io.question("How long (in seconds) to perform scan? "),
    );
  }
  app.run(duration, dataRateString);
  io.close();
}

main();

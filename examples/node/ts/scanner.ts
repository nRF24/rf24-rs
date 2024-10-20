import * as readline from "readline/promises";
import * as fs from "fs";
import * as timer from "timers/promises";
import { RF24, DataRate, FifoState } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

type AppState = {
  radio: RF24;
};

const CHANNELS = 126;

console.log(module.filename);
console.log(
  "!!!Make sure the terminal is wide enough for 126 characters on 1 line." +
    " If this line is wrapped, then the output will look bad!",
);

export async function setup(): Promise<AppState> {
  // The radio's CE Pin uses a GPIO number.
  // On Linux, consider the device path `/dev/gpiochip<N>`:
  //   - `<N>` is the gpio chip's identifying number.
  //     Using RPi4 (or earlier), this number is `0` (the default).
  //     Using the RPi5, this number is actually `4`.
  // The radio's CE pin must connected to a pin exposed on the specified chip.
  const CE_PIN = 22; // for GPIO22
  // try detecting RPi5 first; fall back to default
  const DEV_GPIO_CHIP = fs.existsSync("/dev/gpiochip4") ? 4 : 0;

  // The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
  // On Linux, consider the device path `/dev/spidev<a>.<b>`:
  //   - `<a>` is the SPI bus number (defaults to `0`)
  //   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
  const CSN_PIN = 0; // aka CE0 for SPI bus 0 (/dev/spidev0.0)

  // create a radio object for the specified hard ware config:
  const radio = new RF24(CE_PIN, CSN_PIN, {
    devGpioChip: DEV_GPIO_CHIP,
  });

  // initialize the nRF24L01 on the spi bus
  radio.begin();

  // This is the worst possible configuration.
  // The intention here is to pick up as much noise as possible.
  radio.setAddressLength(2);

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
    radio.openRxPipe(pipe, address[pipe]);
  }

  // Ask user for desired data rate (default to 1 Mbps)
  const dRatePrompt =
    "Select the desired DataRate: (defaults to 1 Mbps)\n" +
    "1. 1 Mbps\n2. 2 Mbps\n3. 250 Kbps\n";
  const answer = parseInt(await io.question(dRatePrompt));
  const index = isNaN(answer) ? 0 : answer;

  if (index == 2) {
    radio.setDataRate(DataRate.Mbps2);
    console.log(`Data Rate is 2 Mbps`);
  }
  if (index == 3) {
    radio.setDataRate(DataRate.Kbps250);
    console.log(`Data Rate is 250 Kbps`);
  } else {
    radio.setDataRate(DataRate.Mbps1);
    console.log(`Data Rate is 1 Mbps`);
  }

  return {
    radio: radio,
  };
}

/**
 * Prints the vertical header for all the channels
 */
export function printHeader() {
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
export async function scan(example: AppState, duration: number | null) {
  printHeader();
  const caches = [];
  for (let i = 0; i < CHANNELS; i++) {
    caches.push(0);
  }
  let sweeps = 0;
  let channel = 0;

  const timeout = Date.now() + (duration || 30) * 1000;
  while (Date.now() < timeout) {
    example.radio.setChannel(channel);
    example.radio.startListening();
    await timer.setTimeout(0.13); // needs to be at least 130 microseconds
    const rpd = example.radio.rpd;
    example.radio.stopListening();
    const foundSignal = example.radio.available();

    caches[channel] += Number(foundSignal || rpd || example.radio.rpd);

    if (foundSignal) {
      example.radio.flushRx(); // discard any packets (noise) saved in RX FIFO
    }
    const total = caches[channel];
    process.stdout.write(total > 0 ? total.toString(16) : "-");

    channel += 1;
    let endl = false;
    if (channel >= CHANNELS) {
      channel = 0;
      sweeps += 1;
    }
    if (sweeps > 15) {
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
export function noise(example: AppState, duration: number | null) {
  const timeout = Date.now() + (duration || 10) * 1000;
  example.radio.startListening();
  while (
    example.radio.isListening ||
    example.radio.getFifoState(false) != FifoState.Empty
  ) {
    const payload = example.radio.read();
    const hexArray = [];
    for (let i = 0; i < payload.length; i++) {
      hexArray.push(payload[i].toString(16).padStart(2, "0"));
    }
    console.log(hexArray.join(" "));
    if (Date.now() > timeout && example.radio.isListening) {
      example.radio.stopListening();
    }
  }
}

/**
 * This function prompts the user and performs the specified role for the radio.
 */
export async function setRole(example: AppState): Promise<boolean> {
  const prompt =
    "*** Enter 'S' to scan\n" +
    "*** Enter 'N' to print noise\n" +
    "*** Enter 'Q' to quit\n";
  const input = (await io.question(prompt)).split(" ");
  let param: number | null = null;
  if (input.length > 1) {
    param = Number(input[1]);
  }
  switch (input[0].charAt(0).toLowerCase()) {
    case "s":
      await scan(example, param);
      return true;
    case "n":
      noise(example, param);
      return true;
    default:
      console.log(`'${input[0].charAt(0)}' is an unrecognized input`);
      return true;
    case "q":
      example.radio.powerDown();
      return false;
  }
}

export async function main() {
  const example = await setup();
  while (await setRole(example));
  io.close();
  example.radio.powerDown();
}

main();

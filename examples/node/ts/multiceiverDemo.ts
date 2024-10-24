import * as readline from "readline/promises";
import * as fs from "fs";
import * as timer from "timers/promises";
import { RF24, PaLevel } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

type AppState = {
  radio: RF24;
  payload: Buffer;
};

// declare the addresses for all transmitting nRF24L01 nodes
const addresses = [
  Buffer.from([0x78, 0x78, 0x78, 0x78, 0x78]),
  Buffer.from([0xf1, 0xb6, 0xb5, 0xb4, 0xb3]),
  Buffer.from([0xcd, 0xb6, 0xb5, 0xb4, 0xb3]),
  Buffer.from([0xa3, 0xb6, 0xb5, 0xb4, 0xb3]),
  Buffer.from([0x0f, 0xb6, 0xb5, 0xb4, 0xb3]),
  Buffer.from([0x05, 0xb6, 0xb5, 0xb4, 0xb3]),
];

console.log(module.filename);

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

  // create a radio object for the specified hardware config:
  const radio = new RF24(CE_PIN, CSN_PIN, {
    devGpioChip: DEV_GPIO_CHIP,
  });

  // initialize the nRF24L01 on the spi bus
  radio.begin();

  // set the Power Amplifier level to -12 dBm since this test example is
  // usually run with nRF24L01 transceivers in close proximity of each other
  radio.setPaLevel(PaLevel.Low); // PaLevel.Max is default

  // To save time during transmission, we'll set the payload size to be only what
  // we need. A 32-bit integer value occupies 4 bytes in memory (using little-endian).
  // We'll be transmitting 2 integers for each payload.
  const payloadLength = 8;
  radio.setPayloadLength(payloadLength);
  // we'll use a DataView object to store our float number into a bytearray buffer
  const payload = Buffer.alloc(payloadLength);
  payload.writeInt32LE(0, 0);
  payload.writeInt32LE(0, 4);

  return { radio: radio, payload: payload };
}

/**
 * The transmitting node's behavior.
 * @param nodeNumber The ID number for this node as a transmitter
 * @param count The number of payloads to send
 */
export async function master(
  app: AppState,
  nodeNumber: number | null,
  count: number | null,
) {
  app.radio.stopListening();
  // clamp node ID to radio's number of pipes for this example
  const id = Math.max(Math.min(nodeNumber || 0, 5), 0);

  // According to the datasheet, the auto-retry features's delay value should
  // be "skewed" to allow the RX node to receive 1 transmission at a time.
  // So, use varying delay between retry attempts and 15 (at most) retry attempts
  const delay = ((id * 3) % 12) + 3;
  const retryCount = 15;
  app.radio.setAutoRetries(delay, retryCount); // max value is 15 for both args

  // set the TX address to the address of the base station.
  app.radio.openTxPipe(addresses[id]);

  if (app.payload.readInt32LE(0) != id) {
    // if node ID has changed since last call to master() (or setup())
    app.payload.writeInt32LE(id, 0); // set this node's ID in offset 0
    app.payload.writeInt32LE(0, 4); // reset payload count
  }

  for (let i = 0; i < (count || 5); i++) {
    const counter = app.payload.readInt32LE(4);
    // set payload's unique ID
    app.payload.writeInt32LE(counter < 0xffff ? counter + 1 : 0, 4);
    const start = process.hrtime.bigint();
    const result = app.radio.send(app.payload);
    const end = process.hrtime.bigint();
    if (result) {
      const elapsed = (end - start) / BigInt(1000);
      console.log(`Transmission successful! Time to Transmit: ${elapsed} us`);
    } else {
      console.log("Transmission failed or timed out!");
    }
    await timer.setTimeout(1000);
  }
}

/**
 * The receiving node's behavior.
 * @param duration The timeout duration (in seconds) to listen after receiving a payload.
 */
export function slave(app: AppState, duration: number | null) {
  // write the addresses to all pipes.
  for (let pipe = 0; pipe < addresses.length; pipe++) {
    app.radio.openRxPipe(pipe, addresses[pipe]);
  }

  app.radio.startListening();
  let timeout = Date.now() + (duration || 6) * 1000;
  while (Date.now() < timeout) {
    const hasRx = app.radio.availablePipe();
    if (hasRx.available) {
      const incoming = app.radio.read();
      app.payload = incoming;
      const nodeId = incoming.readInt32LE(0);
      const payloadId = incoming.readInt32LE(4);
      console.log(
        `Received ${incoming.length} bytes on pipe ${hasRx.pipe} from node `,
        `${nodeId}: payload ${payloadId}`,
      );
      timeout = Date.now() + (duration || 6) * 1000;
    }
  }
  app.radio.stopListening();
}

/**
 * This function prompts the user and performs the specified role for the radio.
 */
export async function setRole(app: AppState): Promise<boolean> {
  const prompt =
    "*** Enter 'R' for receiver role.\n" +
    "*** Enter 'T' for transmitter role.\n" +
    "    Use 'T n' to transmit as node n; n must be in range [0, 5].\n" +
    "*** Enter 'Q' to quit example.\n";
  const input = (await io.question(prompt)).split(" ");
  let param: number | null = null;
  if (input.length > 1) {
    param = Number(input[1]);
  }
  let masterCount: number | null = null;
  if (input.length > 2) {
    masterCount = Number(input[2]);
  }
  switch (input[0].charAt(0).toLowerCase()) {
    case "t":
      await master(app, param, masterCount);
      return true;
    case "r":
      slave(app, param);
      return true;
    default:
      console.log(`'${input[0].charAt(0)}' is an unrecognized input`);
      return true;
    case "q":
      app.radio.powerDown();
      return false;
  }
}

export async function main() {
  const app = await setup();
  while (await setRole(app));
  io.close();
  app.radio.powerDown();
}

main();

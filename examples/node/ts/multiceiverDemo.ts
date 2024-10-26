import * as readline from "readline/promises";
import * as fs from "fs";
import * as timer from "timers/promises";
import { RF24, PaLevel } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;
  payload: Buffer;
  readonly addresses = [
    Buffer.from([0x78, 0x78, 0x78, 0x78, 0x78]),
    Buffer.from([0xf1, 0xb6, 0xb5, 0xb4, 0xb3]),
    Buffer.from([0xcd, 0xb6, 0xb5, 0xb4, 0xb3]),
    Buffer.from([0xa3, 0xb6, 0xb5, 0xb4, 0xb3]),
    Buffer.from([0x0f, 0xb6, 0xb5, 0xb4, 0xb3]),
    Buffer.from([0x05, 0xb6, 0xb5, 0xb4, 0xb3]),
  ];

  constructor() {
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

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    this.radio.setPaLevel(PaLevel.Low); // PaLevel.Max is default

    // To save time during transmission, we'll set the payload size to be only what
    // we need. A 32-bit integer value occupies 4 bytes in memory (using little-endian).
    // We'll be transmitting 2 integers for each payload.
    const payloadLength = 8;
    this.radio.setPayloadLength(payloadLength);
    // we'll use a buffer to store the transmitting node ID and payload ID
    this.payload = Buffer.alloc(payloadLength);
    this.payload.writeInt32LE(0, 0); // init mode ID
    this.payload.writeInt32LE(0, 4); // init payload ID
  }

  /**
   * The transmitting node's behavior.
   * @param nodeNumber The ID number for this node as a transmitter
   * @param count The number of payloads to send
   */
  async tx(nodeNumber?: number, count?: number) {
    this.radio.stopListening();
    // clamp node ID to radio's number of pipes for this example
    const id = Math.max(Math.min(nodeNumber || 0, 5), 0);

    // According to the datasheet, the auto-retry features's delay value should
    // be "skewed" to allow the RX node to receive 1 transmission at a time.
    // So, use varying delay between retry attempts and 15 (at most) retry attempts
    const delay = ((id * 3) % 12) + 3;
    const retryCount = 15;
    this.radio.setAutoRetries(delay, retryCount); // max value is 15 for both args

    // set the TX address to the address of the base station.
    this.radio.openTxPipe(this.addresses[id]);

    if (this.payload.readInt32LE(0) != id) {
      // if node ID has changed since last call to master() (or setup())
      this.payload.writeInt32LE(id, 0); // set this node's ID in offset 0
      this.payload.writeInt32LE(0, 4); // reset payload count
    }

    for (let i = 0; i < (count || 5); i++) {
      const counter = this.payload.readInt32LE(4);
      // set payload's unique ID
      this.payload.writeInt32LE(counter < 0xffff ? counter + 1 : 0, 4);
      const start = process.hrtime.bigint();
      const result = this.radio.send(this.payload);
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
  rx(duration?: number) {
    // write the addresses to all pipes.
    for (let pipe = 0; pipe < this.addresses.length; pipe++) {
      this.radio.openRxPipe(pipe, this.addresses[pipe]);
    }

    this.radio.startListening();
    let timeout = Date.now() + (duration || 6) * 1000;
    while (Date.now() < timeout) {
      const hasRx = this.radio.availablePipe();
      if (hasRx.available) {
        const incoming = this.radio.read();
        this.payload = incoming;
        const nodeId = incoming.readInt32LE(0);
        const payloadId = incoming.readInt32LE(4);
        console.log(
          `Received ${incoming.length} bytes on pipe ${hasRx.pipe} from node `,
          `${nodeId}: payload ${payloadId}`,
        );
        timeout = Date.now() + (duration || 6) * 1000;
      }
    }
    this.radio.stopListening();
  }

  /**
   * This function prompts the user and performs the specified role for the radio.
   */
  async setRole() {
    const prompt =
      "*** Enter 'R' for receiver role.\n" +
      "*** Enter 'T' for transmitter role.\n" +
      "    Use 'T n' to transmit as node n; n must be in range [0, 5].\n" +
      "*** Enter 'Q' to quit example.\n";
    const input = (await io.question(prompt)).split(" ");
    const role = input.shift() || "?";
    const params = Array<number>();
    for (let i = 0; i < input.length; ++i) {
      params.push(Number(input[i]));
    }
    switch (role.charAt(0).toLowerCase()) {
      case "t":
        await this.tx(...params);
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

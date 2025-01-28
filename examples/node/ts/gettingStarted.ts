import * as readline from "readline/promises";
import * as timer from "timers/promises";
import { RF24, PaLevel } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;
  payload: Buffer;

  /**
   * @param radioNumber The number (0 or 1) that identifies the which radio is used in
   * this example that requires 2 radios.
   */
  constructor(radioNumber: number) {
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

    // For this example, we will use different addresses
    // An address needs to be a buffer object (bytearray)
    const address = [Buffer.from("1Node"), Buffer.from("2Node")];

    //set TX address of RX node into the TX pipe
    this.radio.openTxPipe(address[radioNumber]); // always uses pipe 0
    // set RX address of TX node into an RX pipe
    this.radio.openRxPipe(1, address[1 - radioNumber]); // using pipe 1

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    this.radio.paLevel = PaLevel.Low; // PaLevel.Max is default

    // To save time during transmission, we'll set the payload size to be only what
    // we need. A 32-bit float value occupies 4 bytes in memory (using little-endian).
    const payloadLength = 4;
    this.radio.payloadLength = payloadLength;
    // we'll use a DataView object to store our float number into a bytearray buffer
    this.payload = Buffer.alloc(payloadLength);
    this.payload.writeFloatLE(0.0, 0);
  }

  /**
   * The transmitting node's behavior.
   * @param count The number of payloads to send
   */
  async tx(count?: number) {
    this.radio.asTx();
    for (let i = 0; i < (count || 5); ++i) {
      const start = process.hrtime.bigint();
      const result = this.radio.send(this.payload);
      const end = process.hrtime.bigint();
      if (result) {
        const elapsed = (end - start) / BigInt(1000);
        console.log(`Transmission successful! Time to Transmit: ${elapsed} us`);
        this.payload.writeFloatLE(this.payload.readFloatLE(0) + 0.01, 0);
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
    this.radio.asRx();
    let timeout = Date.now() + (duration || 6) * 1000;
    while (Date.now() < timeout) {
      const hasRx = this.radio.availablePipe();
      if (hasRx.available) {
        const incoming = this.radio.read();
        this.payload = incoming;
        const data = incoming.readFloatLE(0);
        console.log(
          `Received ${incoming.length} bytes on pipe ${hasRx.pipe}: ${data}`,
        );
        timeout = Date.now() + (duration || 6) * 1000;
      }
    }
    this.radio.asTx();
  }

  /**
   * This function prompts the user and performs the specified role for the radio.
   */
  async setRole() {
    const prompt =
      "*** Enter 'T' to transmit\n" +
      "*** Enter 'R' to receive\n" +
      "*** Enter 'Q' to quit\n";
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

  // to use different addresses on a pair of radios, we need a variable to
  // uniquely identify which address this radio will use to transmit
  // 0 uses address[0] to transmit, 1 uses address[1] to transmit
  const radioNumber = Number(
    (await io.question(
      "Which radio is this? Enter '1' or '0' (default is '0') ",
    )) == "1",
  );
  console.log(`radioNumber is ${radioNumber}`);

  const app = new App(radioNumber);
  while (await app.setRole());
  io.close();
}

main();

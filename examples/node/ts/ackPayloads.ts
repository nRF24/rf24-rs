/**
 * Simple example of using the nRF24L01 to transmit and
 * retrieve automatic acknowledgment packets with customized payloads.
 *
 * This example is meant to be run on 2 separate nRF24L01 transceivers.
 *
 * See documentation at https://nRF24.github.io/rf24-rs
 */
import * as readline from "readline/promises";
import * as timer from "timers/promises";
import { RF24, PaLevel } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;
  counter: number;

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
    this.radio.asTx(address[radioNumber]); // always uses pipe 0

    // set RX address of TX node into an RX pipe
    this.radio.openRxPipe(1, address[1 - radioNumber]); // using pipe 1

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    this.radio.paLevel = PaLevel.Low; // PaLevel.Max is default

    this.radio.ackPayloads = true;
    this.radio.dynamicPayloads = true;

    this.counter = 0;
  }

  /**
   * The transmitting node's behavior.
   * @param count The number of payloads to send
   */
  async tx(count?: number) {
    this.radio.asTx();
    // we'll use a DataView object to store our string and number into a bytearray buffer
    const outgoing = Buffer.from("Hello \0.");
    for (let i = 0; i < (count || 5); i++) {
      outgoing.writeUint8(this.counter, 7);
      const start = process.hrtime.bigint();
      const result = this.radio.send(outgoing);
      const end = process.hrtime.bigint();
      if (result) {
        const elapsed = (end - start) / BigInt(1000);
        process.stdout.write(
          `Transmission successful! Time to Transmit: ${elapsed} us. Sent: ` +
            `${outgoing.subarray(0, 6).toString("utf-8")}${this.counter} `,
        );
        this.counter += 1;
        if (this.radio.available()) {
          const incoming = this.radio.read();
          const counter = incoming.readUint8(7);
          console.log(
            ` Received: ${incoming.subarray(0, 6).toString("utf-8")}${counter}`,
          );
        } else {
          console.log("Received an empty ACK packet");
        }
      } else {
        console.log("Transmission failed or timed out!");
      }
      await timer.setTimeout(1000);
    }

    // recommended behavior is to keep in TX mode while idle
    this.radio.asTx(); // enter inactive TX mode
  }

  /**
   * The receiving node's behavior.
   * @param duration The timeout duration (in seconds) to listen after receiving a payload.
   */
  rx(duration?: number) {
    this.radio.asRx();
    // we'll use a DataView object to store our string and number into a bytearray buffer
    const outgoing = Buffer.from("World \0.");
    outgoing.writeUint8(this.counter, 7);
    this.radio.writeAckPayload(1, outgoing);
    let timeout = Date.now() + (duration || 6) * 1000;
    while (Date.now() < timeout) {
      const hasRx = this.radio.availablePipe();
      if (hasRx.available) {
        const incoming = this.radio.read();
        const counter = incoming.readUint8(7);
        console.log(
          `Received ${incoming.length} bytes on pipe ${hasRx.pipe}: ` +
            `${incoming.subarray(0, 6).toString("utf-8")}${counter} Sent: ` +
            `${outgoing.subarray(0, 6).toString("utf-8")}${this.counter}`,
        );
        this.counter = counter;
        outgoing.writeUint8(counter + 1, 7);
        this.radio.writeAckPayload(1, outgoing);
        timeout = Date.now() + (duration || 6) * 1000;
      }
    }

    // recommended behavior is to keep in TX mode while idle
    this.radio.asTx(); // enter inactive TX mode
    // `asTx()` also flushes any unused ACK payloads in the TX FIFO
    // when ACK payloads are enabled
  }

  /**
   * This function prompts the user and performs the specified role for the radio.
   */
  async setRole() {
    const prompt =
      "*** Enter 'T' to transmit\n" +
      "*** Enter 'R' to receive\n" +
      "*** Enter 'Q' to quit\n";
    io.resume();
    const input = (await io.question(prompt)).split(" ");
    io.pause();
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

  io.resume();
  // to use different addresses on a pair of radios, we need a variable to
  // uniquely identify which address this radio will use to transmit
  // 0 uses address[0] to transmit, 1 uses address[1] to transmit
  const radioNumber = Number(
    (await io.question(
      "Which radio is this? Enter '1' or '0' (default is '0') ",
    )) == "1",
  );
  console.log(`radioNumber is ${radioNumber}`);
  io.pause();

  const app = new App(radioNumber);
  while (await app.setRole());
  io.close();
}

main();

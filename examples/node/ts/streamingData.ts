/**
 * This example demonstrates how to
 * transmit multiple payloads as a stream of data.
 *
 * This example is meant to be run on 2 separate nRF24L01 transceivers.
 *
 * Any transmission failures will be retried.
 * If the number of failures exceeds 100, then the example aborts
 * transmitting the stream.
 *
 * See documentation at https://nRF24.github.io/rf24-rs
 */
import * as readline from "readline/promises";
import { RF24, PaLevel, FifoState } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;

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
  }

  makePayload(size: number, count: number): Buffer {
    // prefix each payload with a letter to indicate which payloads were lost (if any)
    const prefix = count + (count < 26 ? 65 : 71);
    let payload = String.fromCharCode(prefix);
    const middleByte = Math.abs((size - 1) / 2 - count);
    for (let j = 0; j < size - 1; j++) {
      const byte =
        Boolean(j >= (size - 1) / 2 + middleByte) ||
        Boolean(j < (size - 1) / 2 - middleByte);
      payload += String.fromCharCode(Number(byte) + 48);
    }
    return Buffer.from(payload);
  }

  /**
   * The transmitting node's behavior.
   * @param count The number of streams to send
   * @param size The number of payloads (and the payloads' size) in each stream.
   */
  async tx(count?: number, size?: number) {
    // minimum stream size should be at least 6 payloads for this example.
    const payloadSize = Math.max(Math.min(size || 32, 32), 6);

    // save on transmission time by setting the radio to only transmit the
    // number of bytes we need to transmit
    this.radio.payloadLength = payloadSize; // default is the maximum 32 bytes

    this.radio.asTx(); // put radio into TX mode
    for (let cnt = 0; cnt < (count || 1); cnt++) {
      // for each stream

      let failures = 0;
      const start = Date.now();
      for (let i = 0; i < payloadSize; i++) {
        const buf = this.makePayload(payloadSize, i);
        // for each payload in stream
        while (!this.radio.write(buf)) {
          // upload to TX FIFO failed because TX FIFO is full.
          // check status flags
          const flags = this.radio.getStatusFlags();
          if (flags.txDf) {
            // a transmission failed
            failures += 1; // increment manual retry count

            // we need to reset the txDf flag and the radio's CE pin
            this.radio.cePin(false);
            // NOTE the next call to `write()` will
            // this.radio.clearStatusFlags(); // reset the txDf flag
            // this.radio.cePin(true); // restart transmissions
          }
          if (failures > 49) {
            break; // prevent an infinite loop
          }
        }
        if (failures > 49) {
          // too many failures detected
          console.log("Make sure other node is listening. Aborting stream");
          break; // receiver radio seems unresponsive
        }
      }
      // wait for radio to finish transmitting everything in the TX FIFO
      while (
        failures < 49 &&
        this.radio.getFifoState(true) != FifoState.Empty
      ) {
        // getFifoState() also update()s the StatusFlags
        const flags = this.radio.getStatusFlags();
        if (flags.txDf) {
          failures += 1;
          // we need to reset the txDf flag and the radio's CE pin
          this.radio.cePin(false);
          // do this manually because we're done calling `write()`
          this.radio.clearStatusFlags(); // reset the txDf flag
          this.radio.cePin(true); // restart transmissions
        }
      }
      const end = Date.now();
      console.log(
        `Transmission took ${end - start} ms with ${failures} failures detected`,
      );
    }

    // recommended behavior is to keep in TX mode while idle
    this.radio.asTx(); // enter inactive TX mode
  }

  /**
   * The receiving node's behavior.
   * @param duration The timeout duration (in seconds) to listen after receiving a payload.
   * @param size The number of bytes in each payload
   */
  rx(duration?: number, size?: number) {
    this.radio.payloadLength = Math.max(Math.min(size || 32, 32), 6);
    let count = 0;
    this.radio.asRx();
    let timeout = Date.now() + (duration || 6) * 1000;
    while (Date.now() < timeout) {
      if (this.radio.available()) {
        const incoming = this.radio.read();
        count += 1;
        console.log(`Received: ${incoming.toString()} = ${count}`);
        timeout = Date.now() + (duration || 6) * 1000;
      }
    }

    // recommended behavior is to keep in TX mode while idle
    this.radio.asTx(); // enter inactive TX mode
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

import * as readline from "readline/promises";
import * as fs from "fs";
import { RF24, PaLevel, FifoState } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;

  constructor(radioNumber: number) {
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

    // For this example, we will use different addresses
    // An address needs to be a buffer object (bytearray)
    const address = [Buffer.from("1Node"), Buffer.from("2Node")];

    //set TX address of RX node into the TX pipe
    this.radio.openTxPipe(address[radioNumber]); // always uses pipe 0
    // set RX address of TX node into an RX pipe
    this.radio.openRxPipe(1, address[1 - radioNumber]); // using pipe 1

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    this.radio.setPaLevel(PaLevel.Low); // PaLevel.Max is default
  }

  makePayloads(size: number) {
    const arr = Array<Buffer>();
    for (let i = 0; i < size; i++) {
      // prefix each payload with a letter to indicate which payloads were lost (if any)
      const prefix = i + (i < 26 ? 65 : 71);
      let payload = String.fromCharCode(prefix);
      const middleByte = Math.abs((size - 1) / 2 - i);
      for (let j = 0; j < size - 1; j++) {
        const byte =
          Boolean(j >= (size - 1) / 2 + middleByte) ||
          Boolean(j < (size - 1) / 2 - middleByte);
        payload += String.fromCharCode(Number(byte) + 48);
      }
      arr.push(Buffer.from(payload));
    }
    return arr;
  }

  /**
   * The transmitting node's behavior.
   * @param count The number of streams to send
   * @param size The number of payloads (and the payloads' size) in each stream.
   */
  async tx(count?: number, size?: number) {
    // minimum stream size should be at least 6 payloads for this example.
    const payloadSize = Math.max(Math.min(size || 32, 32), 6);
    const payloads = this.makePayloads(payloadSize);
    // save on transmission time by setting the radio to only transmit the
    // number of bytes we need to transmit
    this.radio.setPayloadLength(payloadSize); // default is the maximum 32 bytes

    this.radio.stopListening(); // put radio into TX mode
    for (let cnt = 0; cnt < (count || 1); cnt++) {
      // for each stream

      let failures = 0;
      const start = Date.now();
      for (let bufIndex = 0; bufIndex < payloadSize; bufIndex++) {
        // for each payload in stream
        while (!this.radio.write(payloads[bufIndex])) {
          // upload to TX FIFO failed because TX FIFO is full.
          // check status flags
          this.radio.update();
          const flags = this.radio.getStatusFlags();
          if (flags.txDf) {
            // transmission failed
            this.radio.rewrite(); // resets txDf flag and reuses top level of TX FIFO
            failures += 1; // increment manual retry count
            if (failures > 99) {
              // too many failures detected
              break; // prevent infinite loop
            }
          }
        }
        if (failures > 99 && bufIndex < 7 && cnt < 2) {
          this.radio.flushTx();
          break; // receiver radio seems unresponsive
        }
      }
      // wait for radio to finish transmitting everything in the TX FIFO
      while (
        this.radio.getFifoState(true) != FifoState.Empty &&
        failures < 99
      ) {
        // getFifoState() also update()s the StatusFlags
        const flags = this.radio.getStatusFlags();
        if (flags.txDf) {
          failures += 1;
          this.radio.rewrite();
        }
      }
      const end = Date.now();
      console.log(
        `Transmission took ${end - start} ms with ${failures} failures detected`,
      );
    }
    this.radio.stopListening(); // ensure radio exits active TX mode
  }

  /**
   * The receiving node's behavior.
   * @param duration The timeout duration (in seconds) to listen after receiving a payload.
   * @param size The number of bytes in each payload
   */
  rx(duration?: number, size?: number) {
    this.radio.setPayloadLength(Math.max(Math.min(size || 32, 32), 6));
    let count = 0;
    this.radio.startListening();
    let timeout = Date.now() + (duration || 6) * 1000;
    while (Date.now() < timeout) {
      if (this.radio.available()) {
        const incoming = this.radio.read();
        count += 1;
        console.log(`Received: ${incoming.toString()} = ${count}`);
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

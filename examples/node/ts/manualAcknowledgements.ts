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
  counter: number;
};

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

  // For this example, we will use different addresses
  // An address needs to be a buffer object (bytearray)
  const address = [Buffer.from("1Node"), Buffer.from("2Node")];

  // to use different addresses on a pair of radios, we need a variable to
  // uniquely identify which address this radio will use to transmit
  // 0 uses address[0] to transmit, 1 uses address[1] to transmit
  const radioNumber = Number(
    (await io.question(
      "Which radio is this? Enter '1' or '0' (default is '0') ",
    )) == "1",
  );
  console.log(`radioNumber is ${radioNumber}`);
  //set TX address of RX node into the TX pipe
  radio.openTxPipe(address[radioNumber]); // always uses pipe 0
  // set RX address of TX node into an RX pipe
  radio.openRxPipe(1, address[1 - radioNumber]); // using pipe 1

  // set the Power Amplifier level to -12 dBm since this test example is
  // usually run with nRF24L01 transceivers in close proximity of each other
  radio.setPaLevel(PaLevel.Low); // PaLevel.Max is default

  // To save time during transmission, we'll set the payload size to be only what
  // we need.
  //
  // we only need 1 unsigned byte (for payload ID) + 7 more bytes for the payload message
  const payloadLength = 8;
  radio.setPayloadLength(payloadLength);

  return { radio: radio, counter: 0 };
}

/**
 * The transmitting node's behavior.
 * @param count The number of payloads to send
 */
export async function master(app: AppState, count: number | null) {
  app.radio.stopListening();
  // we'll use a DataView object to store our string and number into a bytearray buffer
  const outgoing = Buffer.from("Hello \0.");
  for (let i = 0; i < (count || 5); i++) {
    outgoing.writeUint8(app.counter, 7);
    const start = process.hrtime.bigint();
    if (app.radio.send(outgoing)) {
      let gotResponse = false;
      app.radio.startListening();
      const responseTimeout = Date.now() + 200; // wait at most 200 milliseconds
      while (Date.now() < responseTimeout) {
        if (app.radio.available()) {
          gotResponse = true;
          break;
        }
      }
      app.radio.stopListening();
      const end = process.hrtime.bigint();
      const elapsed = (end - start) / BigInt(1000);
      process.stdout.write(
        `Transmission successful! Time to Transmit: ${elapsed} us. Sent: ` +
          `${outgoing.subarray(0, 6).toString()}${app.counter} `,
      );
      app.counter += 1;
      if (gotResponse) {
        const incoming = app.radio.read();
        const counter = incoming.readUint8(7);
        console.log(
          `Received: ${incoming.subarray(0, 6).toString()}${counter}`,
        );
      } else {
        console.log("Received no response");
      }
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
  app.radio.startListening();
  // we'll use a DataView object to store our string and number into a bytearray buffer
  const outgoing = Buffer.from("World \0.");
  let timeout = Date.now() + (duration || 6) * 1000;
  while (Date.now() < timeout) {
    const hasRx = app.radio.availablePipe();
    if (hasRx.available) {
      const incoming = app.radio.read();
      app.counter = incoming.readUint8(7);
      outgoing.writeUint8(app.counter, 7);
      app.radio.stopListening();
      app.radio.write(outgoing);
      let responseResult = false;
      const responseTimeout = Date.now() + 150; // try to respond for 150 milliseconds
      while (Date.now() < responseTimeout) {
        app.radio.update();
        const flags = app.radio.getStatusFlags();
        if (flags.txDs) {
          responseResult = true;
          break;
        }
        if (flags.txDf) {
          app.radio.rewrite();
        }
      }
      app.radio.startListening();
      process.stdout.write(
        `Received ${incoming.length} bytes on pipe ${hasRx.pipe}: ` +
          `${incoming.subarray(0, 6).toString()}${app.counter} `,
      );
      if (responseResult) {
        console.log(
          `Sent: ${outgoing.subarray(0, 6).toString()}${app.counter}`,
        );
      } else {
        app.radio.flushTx();
        console.log("Response failed");
      }
      timeout = Date.now() + (duration || 6) * 1000;
    }
  }
  app.radio.stopListening(); // flushes TX FIFO when ACK payloads are enabled
}

/**
 * This function prompts the user and performs the specified role for the radio.
 */
export async function setRole(app: AppState): Promise<boolean> {
  const prompt =
    "*** Enter 'T' to transmit\n" +
    "*** Enter 'R' to receive\n" +
    "*** Enter 'Q' to quit\n";
  const input = (await io.question(prompt)).split(" ");
  let param: number | null = null;
  if (input.length > 1) {
    param = Number(input[1]);
  }
  switch (input[0].charAt(0).toLowerCase()) {
    case "t":
      await master(app, param);
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

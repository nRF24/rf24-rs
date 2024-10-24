import * as readline from "readline/promises";
import * as fs from "fs";
import { RF24, PaLevel, FifoState } from "@rf24/rf24";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

type AppState = {
  radio: RF24;
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

  return { radio: radio };
}

export function makePayloads(size: number): Array<Buffer> {
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
export async function master(
  app: AppState,
  count: number | null,
  size: number | null,
) {
  // minimum stream size should be at least 6 payloads for this example.
  const payloadSize = Math.max(Math.min(size || 32, 32), 6);
  const payloads = makePayloads(payloadSize);
  // save on transmission time by setting the radio to only transmit the
  // number of bytes we need to transmit
  app.radio.setPayloadLength(payloadSize); // default is the maximum 32 bytes

  app.radio.stopListening(); // put radio into TX mode
  for (let cnt = 0; cnt < (count || 1); cnt++) {
    // for each stream

    let failures = 0;
    const start = Date.now();
    for (let bufIndex = 0; bufIndex < payloadSize; bufIndex++) {
      // for each payload in stream
      while (!app.radio.write(payloads[bufIndex])) {
        // upload to TX FIFO failed because TX FIFO is full.
        // check status flags
        app.radio.update();
        const flags = app.radio.getStatusFlags();
        if (flags.txDf) {
          // transmission failed
          app.radio.rewrite(); // resets txDf flag and reuses top level of TX FIFO
          failures += 1; // increment manual retry count
          if (failures > 99) {
            // too many failures detected
            break; // prevent infinite loop
          }
        }
      }
      if (failures > 99 && bufIndex < 7 && cnt < 2) {
        app.radio.flushTx();
        break; // receiver radio seems unresponsive
      }
    }
    // wait for radio to finish transmitting everything in the TX FIFO
    while (app.radio.getFifoState(true) != FifoState.Empty && failures < 99) {
      // getFifoState() also update()s the StatusFlags
      const flags = app.radio.getStatusFlags();
      if (flags.txDf) {
        failures += 1;
        app.radio.rewrite();
      }
    }
    const end = Date.now();
    console.log(
      `Transmission took ${end - start} ms with ${failures} failures detected`,
    );
  }
  app.radio.stopListening(); // ensure radio exits active TX mode
}

/**
 * The receiving node's behavior.
 * @param duration The timeout duration (in seconds) to listen after receiving a payload.
 * @param size The number of bytes in each payload
 */
export function slave(
  app: AppState,
  duration: number | null,
  size: number | null,
) {
  app.radio.setPayloadLength(Math.max(Math.min(size || 32, 32), 6));
  let count = 0;
  app.radio.startListening();
  let timeout = Date.now() + (duration || 6) * 1000;
  while (Date.now() < timeout) {
    if (app.radio.available()) {
      const incoming = app.radio.read();
      count += 1;
      console.log(`Received: ${incoming.toString()} = ${count}`);
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
    "*** Enter 'T' to transmit\n" +
    "*** Enter 'R' to receive\n" +
    "*** Enter 'Q' to quit\n";
  const input = (await io.question(prompt)).split(" ");
  let param: number | null = null;
  if (input.length > 1) {
    param = Number(input[1]);
  }
  let size = null;
  if (input.length > 2) {
    size = Number(input[2]);
  }
  switch (input[0].charAt(0).toLowerCase()) {
    case "t":
      await master(app, param, size);
      return true;
    case "r":
      slave(app, param, size);
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

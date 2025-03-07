/**
 * Simple example of detecting (and verifying) the IRQ (interrupt) pin on the
 * nRF24L01.
 *
 * This example is meant to be run on 2 separate nRF24L01 transceivers.
 *
 * This example requires gpiod lib to monitor the radio's IRQ pin.
 *
 * See documentation at https://nRF24.github.io/rf24-rs
 */
import * as readline from "readline/promises";
import * as timer from "timers/promises";
import { RF24, PaLevel, FifoState } from "@rf24/rf24";
import { Default as GpioPin, Pin } from "opengpio";

const io = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export class App {
  radio: RF24;
  // An iterator used during the TX role.
  plIterator: number;
  // The GPIO pin connected to the radio's IRQ pin.
  irqPin: Pin;

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

    this.radio.ackPayloads = true;
    this.radio.dynamicPayloads = true;

    this.plIterator = 0;

    this.irqPin = GpioPin.input({ chip: 0, line: 24 }); // using GPIO24 exposed via /dev/gpiochip0
  }

  private waitForIRQ(timeout: number): boolean {
    const endTime = Date.now() + timeout * 1000;
    let eventOccurred = false;
    while (Date.now() < endTime && !eventOccurred) {
      eventOccurred = this.irqPin.value == false;
    }
    if (!eventOccurred) {
      console.log(`\tInterrupt event not detected for ${timeout} seconds!`);
      return false;
    }
    return true;
  }

  /**
   * This function is called when IRQ pin is detected active LOW
   */
  private interruptHandler() {
    console.log("\tIRQ pin went active LOW.");
    this.radio.update();
    const flags = this.radio.getStatusFlags(); // update IRQ status flags
    console.log(`\t${flags}`);
    if (this.plIterator == 0) {
      console.log("'data ready' event test", flags.rxDr ? "passed" : "failed");
    } else if (this.plIterator == 1) {
      console.log("'data sent' event test", flags.txDs ? "passed" : "failed");
    } else if (this.plIterator == 2) {
      console.log("'data fail' event test", flags.txDf ? "passed" : "failed");
    }
    this.radio.clearStatusFlags();
  }

  /**
   * Transmits 4 times and reports results
   *
   * 1. successfully receive ACK payload first
   * 2. successfully transmit on second
   * 3. send a third payload to fill RX node's RX FIFO
   *    (supposedly making RX node unresponsive)
   * 4. intentionally fail transmit on the fourth
   */
  async tx() {
    const txPayloads = [
      Buffer.from("Ping "),
      Buffer.from("Pong "),
      Buffer.from("Radio"),
      Buffer.from("FAIL!"),
    ];
    this.radio.asTx(); // put radio in TX mode

    // on data ready test
    console.log("\nConfiguring IRQ pin to only ignore 'on data sent' event");
    this.radio.setStatusFlags({ rxDr: true, txDs: false, txDf: true });
    console.log("    Pinging slave node for an ACK payload...");
    this.plIterator = 0;
    this.radio.write(txPayloads[0]);
    if (this.waitForIRQ(5)) {
      this.interruptHandler();
    }

    // on "data sent" test
    console.log("\nConfiguring IRQ pin to only ignore 'on data ready' event");
    this.radio.setStatusFlags({ rxDr: false, txDs: true, txDf: true });
    console.log("    Pinging slave node again...");
    this.plIterator = 1;
    this.radio.write(txPayloads[1]);
    if (this.waitForIRQ(5)) {
      this.interruptHandler();
    }

    // trigger slave node to exit by filling the slave node's RX FIFO
    console.log("\nSending one extra payload to fill RX FIFO on slave node.");
    console.log("Disabling IRQ pin for all events.");
    this.radio.setStatusFlags({});
    if (this.radio.send(txPayloads[2])) {
      console.log("Slave node should not be listening anymore.");
    } else {
      console.log("Slave node was unresponsive.");
    }
    this.radio.clearStatusFlags();

    // on "data fail" test
    console.log("\nConfiguring IRQ pin to go active for all events.");
    this.radio.setStatusFlags();
    console.log("    Sending a ping to inactive slave node...");
    this.radio.flushTx(); // just in case any previous tests failed
    this.plIterator = 2;
    this.radio.write(txPayloads[3]);
    if (this.waitForIRQ(5)) {
      this.interruptHandler();
    }
    this.radio.flushTx(); // flush artifact payload in TX FIFO from last test
    // all 3 ACK payloads received were 4 bytes each, and RX FIFO is full
    // so, fetching 12 bytes from the RX FIFO also flushes RX FIFO
    console.log("\nComplete RX FIFO:", this.radio.read(12).toString("utf-8"));
  }

  /**
   * The receiving node's behavior.
   * @param duration The timeout duration (in seconds) to listen after receiving a payload.
   */
  rx(duration?: number) {
    // load ACK payloads into TX FIFO
    const ackPayloads = [
      Buffer.from("Yak "),
      Buffer.from("Back"),
      Buffer.from(" Ack"),
    ];
    for (const ack of ackPayloads) {
      this.radio.writeAckPayload(1, ack);
    }

    // the "data ready" event will trigger in RX mode
    // the "data sent" or "data fail" events will trigger when we
    // receive with ACK payloads enabled (& loaded in TX FIFO)
    console.log("\nDisabling IRQ pin for all events.");
    this.radio.setStatusFlags({ rxDr: false, txDs: false, txDf: false });

    this.radio.asRx();
    const timeout = Date.now() + (duration || 6) * 1000;
    while (
      Date.now() < timeout &&
      this.radio.getFifoState(false) != FifoState.Full
    ) {
      // wait for RX FIFO to fill up or until timeout is reached
    }
    timer.setTimeout(500); // wait for last ACK payload to transmit

    // exit RX mode
    this.radio.asTx(); // also clears the TX FIFO when ACK payloads are enabled

    if (this.radio.available()) {
      // If RX FIFO is not empty (timeout did not occur).
      // All 3 payloads received were 5 bytes each, and RX FIFO is full.
      // So, fetching 15 bytes from the RX FIFO also flushes RX FIFO.
      console.log("Complete RX FIFO:", this.radio.read(15).toString("utf-8"));
    }
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
        await this.tx();
        return true;
      case "r":
        await this.rx(...params);
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

import * as readline from 'readline/promises';
import * as fs from 'fs';
import { RF24, PaLevel } from '@rf24/rf24';

const io = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

function delay(ms: number) {
    return new Promise(function (resolve) {
        return setTimeout(resolve, ms);
    });
}

type ExampleStates = {
    radio: RF24,
    payload: DataView
}

async function setup(): Promise<ExampleStates> {
    // The radio's CE Pin uses a GPIO number.
    // On Linux, consider the device path `/dev/gpiochip<N>`:
    //   - `<N>` is the gpio chip's identifying number.
    //     Using RPi4 (or earlier), this number is `0` (the default).
    //     Using the RPi5, this number is actually `4`.
    // The radio's CE pin must connected to a pin exposed on the specified chip.
    const CE_PIN = 22; // for GPIO22
    // try detecting RPi5 first; fall back to default
    const DEV_GPIO_CHIP = fs.existsSync('/dev/gpiochip4') ? 4 : 0;

    // The radio's CSN Pin corresponds the SPI bus's CS pin (aka CE pin).
    // On Linux, consider the device path `/dev/spidev<a>.<b>`:
    //   - `<a>` is the SPI bus number (defaults to `0`)
    //   - `<b>` is the CSN pin (must be unique for each device on the same SPI bus)
    const CSN_PIN = 0; // aka CE0 for SPI bus 0 (/dev/spidev0.0)

    // create a radio object for the specified hard ware config:
    const radio = new RF24(CE_PIN, CSN_PIN, {
        devGpioChip: DEV_GPIO_CHIP
    });

    // initialize the nRF24L01 on the spi bus
    radio.begin();

    // For this example, we will use different addresses
    // An address needs to be a buffer object (bytearray)
    const address = [
        Buffer.from("1Node"),
        Buffer.from("2Node")
    ];

    // to use different addresses on a pair of radios, we need a variable to
    // uniquely identify which address this radio will use to transmit
    // 0 uses address[0] to transmit, 1 uses address[1] to transmit
    const radioNumber = Number(await io.question("Which radio is this? Enter '1' or '0' (default is '0')") == '1');
    console.log('radioNumber is ${radioNumber}');
    //set TX address of RX node into the TX pipe
    radio.openTxPipe(address[1 - radioNumber]); // always uses pipe 0
    // set RX address of TX node into an RX pipe
    radio.openRxPipe(1, address[radioNumber]); // using pipe 1

    // set the Power Amplifier level to -12 dBm since this test example is
    // usually run with nRF24L01 transceivers in close proximity of each other
    radio.setPaLevel(PaLevel.Low); // PaLevel.Max is default

    // To save time during transmission, we'll set the payload size to be only what
    // we need. A 32-bit float value occupies 4 bytes in memory (using little-endian).
    const payloadLength = 4;
    radio.setPayloadLength(payloadLength);
    // we'll use a DataView object to store our float number into a bytearray buffer
    const payload = new DataView(new ArrayBuffer(payloadLength));
    payload.setFloat32(0, 0.0, true); // true means using little endian

    return {radio: radio, payload: payload};
}

/**
 * The transmitting node's behavior.
 * @param count The number of payloads to send
 */
async function master(example: ExampleStates, count: number = 5) {
    example.radio.startListening();
    while (count--) {
        const start = process.hrtime.bigint();
        const result = example.radio.send(Buffer.from(example.payload.buffer));
        const end = process.hrtime.bigint();
        if (result) {
            const elapsed = (end - start) / BigInt(1000);
            console.log('Transmission successful! Time to Transmit: ' + elapsed + ' ms');
            example.payload.setFloat32(0, example.payload.getFloat32(0) + 0.01);
        } else {
            console.log('Transmission failed or timed out!');
        }
        await delay(1000)
    }
}

/**
 * The receiving node's behavior.
 * @param duration The timeout duration (in seconds) to listen after receiving a payload.
 */
function slave(example: ExampleStates, duration: number = 6) {
    example.radio.startListening();
    const time = new Date();
    const timeout = time.getSeconds() + duration;
    while(time.getSeconds() < timeout){
        const hasRx = example.radio.availablePipe();
        if (hasRx.available) {
            const received = example.radio.read();
            example.payload = new DataView(received.buffer);
            const data = example.payload.getFloat32(0);
            console.log('Received ${received.length} bytes on pipe ${hasRx.pipe}: ' + data);
        }
    }
    example.radio.stopListening();
}

/**
 * This function prompts the user and performs the specified role for the radio.
 */
async function setRole(example: ExampleStates): Promise<boolean> {
    const prompt = "*** Enter 'T' to transmit\n" + "*** Enter 'R' to receive\n" + "*** Enter 'Q' to quit\n";
    const input = await io.question(prompt);
    switch (input.toLowerCase()[0]) {
        case 't':
            await master(example);
            return true;
        case 'r':
            slave(example);
            return true;
        default:
            console.log("'${input[0]}' is an unrecognized input");
            return true;
        case 'q':
            example.radio.powerDown();
            return false;
    }
}

async function main() {
    const example = await setup();
    while (setRole(example));
}

main();

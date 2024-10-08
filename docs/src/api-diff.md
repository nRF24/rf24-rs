# Differences in RF24 API conventions

This document will highlight the differences between RF24 API implemented in C++ and this rf24-rs package's API.

There are some important design decisions here.

[traits]: https://doc.rust-lang.org/book/ch10-02-traits.html
[result]: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html#handling-potential-failure-with-result

## STATUS byte exposed

As with our other implementations, the STATUS byte returned on every SPI transaction is cached to a private member. Understanding the meaning of the status byte is publicly exposed via

- `clear_status_flags()`: with parameters to specify which flag(s) should be cleared.
- `get_status_flags()`: has a signature similar to C++ `whatHappened()` but does not clear the flags.
- `set_status_flags()`: similar to C++ `maskIRQ()` except the boolean parameters' meaning is not reversed.

    | lang | only trigger on RX_DR events |
    |:----:|:-----------------------------|
    | C++  | `radio.maskIRQ(false, true, true)` |
    | Rust | `radio.set_status_flags(true, false, false)` |

## No babysitting

To transmit something, RF24 struct offers

- `write()`: non-blocking uploads to TX FIFO.
- `send()`: blocking wrapper around `write()`

There will be no equivalents to C++ `writeBlocking()`, `startFastWrite()`, `writeFast()`, `txStandby()`.
Considering the exposed STATUS byte, these can all be done from the user space (if needed).

Additionally, `send()` does _**not**_ implement a timeout.
Every member function in the `RF24` struct (except the `new()`) returns a [`Result`][result],
so problems with the SPI connections should be detected early in the app lifecycle.
The rustc compiler will warn users about unhandled [`Result`][result]s.

As an alternative, I've been considering an optional `irq_pin` parameter to the constructor.
If specified in user code, then `send()` shall wait for the IRQ pin to become active instead of pinging the radio's STATUS byte over SPI.

> [!TIP]
> Rust does offer a way to overload functions using [traits] (feature akin to C++ templates),
> but it isn't traditionally relied upon in a public API.

## API structure

You'll notice that I used an API structure similar to CircuitPython_nRF24L01.
Under the hood is very much like C++ RF24 lib with respect to radio configuration.

Using rust's [trait][traits] feature, I plan to have an API structured like so

```mermaid
flowchart TD
    subgraph radio
    esb("EsbRadio (trait)") --> RF24
    esb --> nrf51{{RF51}}
    esb --> nrf52{{RF52}}
    esb--> ble{{FakeBle}}
    end

    radio --> net{{RF24Network}}
    net --> mesh{{RF24Mesh}}
```

<!-- markdownlint-disable MD046 -->
!!! info "Graph Legend"

    In the above graph, nodes in angle brackets are not implemented yet.
    This is just how I envision the final result.

This way users can devise their own implementation of the the `EsbRadio` traits and still pass their derivative to the network/mesh implementations provided.

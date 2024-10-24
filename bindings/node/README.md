# `@rf24/rf24`

The node.js binding for the [rf24-rs] project (written in rust).

[rf24-rs]: https://github.com/nRF24/rf24-rs

This package is only functional on Linux machines.
Although, installing this package in non-Linux environments will
provide the typing information used on Linux.

## Install

To install from npmjs.org:

```text
npm install @rf24/rf24
```

To build from source:

```text
yarn install
yarn build:debug
```

## Examples

The examples are written in Typescript and located in the repository's root path "examples/node/ts".
To compile them to Javascript, run the following commands:

```text
yarn install
yarn examples-build
```

Afterwards the Javascript files are located "examples/node/js".
To run them just pass the example file's path to the node interpreter:

```text
node examples/node/js/gettingStarted.js
```

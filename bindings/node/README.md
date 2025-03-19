# `@rf24/rf24`

[rtd-badge]: https://img.shields.io/readthedocs/rf24-rs
[docs]: https://rf24-rs.readthedocs.io/en/latest/node-api/
[rf24-rs]: https://github.com/nRF24/rf24-rs
[npm-badge]: https://img.shields.io/npm/v/%40rf24%2Frf24
[npm-link]: https://www.npmjs.com/package/@rf24/rf24
[node-version]: https://img.shields.io/node/v/%40rf24%2Frf24?color=blue

[![NPM Version][npm-badge]][npm-link]
[![Node.js API][rtd-badge]][docs]
![Node Current][node-version]

The node.js binding for the [rf24-rs] project (written in rust).

This package is only functional on Linux machines.
Although, installing this package in non-Linux environments will
provide the typing information used on Linux.

See the [docs] for more detail about the API.

## Install

To install from npmjs.com:

```text
npm install @rf24/rf24
```

To build from source:

```text
yarn install
yarn build:debug
```

## Examples

The examples are written in Typescript and located in [the repository's root path][rf24-rs] "examples/node/ts".
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

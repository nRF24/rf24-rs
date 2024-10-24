# `rf24-py`

The python binding for the [rf24-rs] project (written in rust).

[rf24-rs]: https://github.com/nRF24/rf24-rs

This package is only functional on Linux machines.
Although, installing this package in non-Linux environments will
provide the typing information used on Linux.

## Install

To install from pypi.org:

```text
pip install rf24-py
```

To build from source:

```text
pip install maturin
maturin dev
```

## Examples

The examples are located in the repository's root path "examples/python".
To run the examples, simply pass the example file's path to the python interpreter:

```text
python examples/python/getting_started.py
```

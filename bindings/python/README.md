# `rf24-py`

[pypi-link]: https://pypi.org/project/rf24-py/
[pypi-badge]: https://img.shields.io/pypi/v/rf24-py
[piwheels-badge]: https://img.shields.io/piwheels/v/rf24-py
[piwheels-link]: https://www.piwheels.org/project/rf24-py/
[rtd-badge]: https://img.shields.io/readthedocs/rf24-rs
[docs]: https://rf24-rs.readthedocs.io/en/latest/python-api/
[rf24-rs]: https://github.com/nRF24/rf24-rs
[py-min-ver]: https://img.shields.io/badge/python->=3.8-blue
[changelog-badge]: https://img.shields.io/badge/keep_a_change_log-v1.1.0-ffec3d
[changelog-link]: https://rf24-rs.readthedocs.io/en/latest/rf24-py-changelog/

[![PyPI - Version][pypi-badge]][pypi-link]
[![PiWheels Version][piwheels-badge]][piwheels-link]
[![Python API][rtd-badge]][docs]
![Minimum Python Version: >=3.8][py-min-ver]
 [![CHANGELOG][changelog-badge]][changelog-link]

The python binding for the [rf24-rs] project (written in rust).

This package is only functional on Linux machines.
Although, installing this package in non-Linux environments will
provide the typing information used on Linux.

See the [docs] for more detail about the API.

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

The examples are located in [the repository's root path][rf24-rs] "examples/python".
To run the examples, simply pass the example file's path to the python interpreter:

```text
python examples/python/getting_started.py
```

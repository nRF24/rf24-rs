# Supplemental documentation

This folder has the documentation sources that are not suitable for rust API documentation.

To build these docs see the [CONTRIBUTING guidelines](../CONTRIBUTING.md#documentation).
In short, the `nur docs --open` command does the following:

```shell
uv sync --group docs
yarn install
# Generate the Node.js binding's API docs:
yarn docs
# Then build and view the docs:
uv run mkdocs serve --config-file docs/mkdocs.yml --open
```

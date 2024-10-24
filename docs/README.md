# Supplemental documentation

This folder has the documentation sources that are not suitable for API documentation.

To build these docs install mkdocs and relevant plugins:

```shell
pip install -r docs/requirements.txt
yarn install
```

Generate the Node.js binding's API docs:

```shell
yarn docs
```

Then build and view the docs using:

```shell
mkdocs serve --config-file docs/mkdocs.yml --open
```

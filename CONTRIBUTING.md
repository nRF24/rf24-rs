# Contribution guidelines

First, thank you for considering a contribution to this project!

This document should help detail the project's expectations about contributions.

## Development tools

This project uses the following tools for development:

- [cargo-llvm-cov] for measuring code coverage.
- [cargo-nextest] as a testing harness.
- [nur] for running common tasks in the development workflow.
- [uv] for managing python virtual environments and dependencies.
- [yarn] for managing javascript dependencies.
  We have checked in the required version of [yarn] to this repository.
  So, really you just need [node.js] installed with `corepack` enabled.

### Optional tools

- [committed] for verifying commit messages conform to [conventional commit] standards.
- [git-cliff] for generating a Changelog(s) and release notes.
- [pre-commit] for sanitizing project files.

[cargo-llvm-cov]: https://crates.io/crates/cargo-llvm-cov
[cargo-nextest]: https://crates.io/crates/cargo-nextest
[nur]: https://crates.io/crates/nur
[uv]: https://docs.astral.sh/uv
[yarn]: https://yarnpkg.com/
[node.js]: https://nodejs.org
[committed]: https://crates.io/crates/committed
[conventional commit]: https://www.conventionalcommits.org
[git-cliff]: https://crates.io/crates/git-cliff
[pre-commit]: https://pre-commit.com

## Submitting patches

Please, please, please open an issue to discuss possible solutions before submitting a Pull Request.
If it is a small patch (ie 1 or 2 lines), then a preemptive issue may not be warranted.
Although, it still helps to first discuss the reason of the small patch in some manor.

Pull Request titles should conform to [conventional commit] standard.
Upon merging the Pull Request, all commits on the feature branch are squashed into a single commit pushed to the default (main) branch.
This is done so [git-cliff] can adequately generate a list of changes when processing a release.

## Code style

This project's CI leverages [pre-commit] to ensure

- [x] line ending all use LF (not CRLF)
- [x] lines have no trailing whitespace
- [x] files end with a blank line
- [x] valid syntax is used in all yaml and toml files
- [x] no large files (greater than 500 kB) are added
- [x] no unknown or misspelled words are present
- [x] python code is properly formatted, linted, and type checked

Normally, [pre-commit] is typically run from a Python virtual environment.
This project uses [uv] to manage a Python virtual environment.
Therefore, [pre-commit] can be run as with a one-line command using [uv] or [nur].

```shell
uv run pre-commit run --all-files
```

<details><summary>Enabling pre-commit hooks automatically</summary>

When creating a commit via `git`, the [pre-commit] hooks can be automatically invoked. This may slow the development workflow a bit and requires a separate command to enable it.

```shell
uv run pre-commit install
```

</details>

Note, [uv] should also build this project's python binding when using `uv run`.

```shell
nur pre-commit
```

By default, the `nur pre-commit` task uses the [uv] command stated above.
Optional arguments are documented and shown in `nur pre-commit -h`.

### Static analysis

Format and linting rust code is done by using `cargo clippy` and `cargo fmt`.
Both commands are performed using a [nur] task for convenience:

```shell
nur lint
```

All python sources are already handled by [pre-commit] hooks.
There is an optional flag to lint and format javascript (and typescript) sources as well:

```shell
nur lint --js
```

The `--js` option invokes `oxlint` and `prettier` via [yarn] scripts.

By default the `nur lint` task focuses on rust sources in `crates/**` and `bindings/**`.
To lint and format the rust examples, we have a separate [nur] task for that:

```shell
nur lint examples
```

## Testing

To collect code coverage, this project uses [cargo-llvm-cov] which natively supports [cargo-nextest].
However, the commands to run [cargo-llvm-cov] and [cargo-nextest] tools in tandem can be lengthy.
We use [nur] to parametrize the various test options (applicable to this project) into tasks.

### Running tests

Unit tests are performed using [cargo-nextest] while coverage is measured by [cargo-llvm-cov].

#### Run the tests

```shell
nur test
```

Optional arguments are documented and shown in `nur test -h`.

<details><summary>Mimicking CI test runs</summary>

The `default` test profile skips tests that are known to run longer than
10 seconds and only shows verbose output for tests that fail.
The `ci` test profile includes slow tests and enables more verbose output.
To enable the `ci` test profile, simply pass `--profile ci` (or `-p ci`)
to the `nur test` command:

```shell
nur test -p ci
```

</details>

#### Generate coverage report

```shell
nut test llvm-cov
```

> [!TIP]
> Additional arguments passed to `nur test llvm-cov` are passed onto [cargo-llvm-cov].
>
> Pass `--open` to automatically open the built coverage report in your default browser.
> Optional arguments are documented and shown in `nur test llvm-cov -h`.

<details><summary>Generating lcov.info</summary>

A "lcov.info" file is uploaded to codecov.
Some developer tooling might also make use of the lcov format (eg. the VS Code ext named "Coverage Gutters").
This lcov.info file can be created with our [nur] task:

```shell
nur test lcov
```

</details>

## Documentation

We host a supplemental set of documentation that aims to

1. Declare helpful information that does not belong in API documentation.
   This can be anything including this guide, CHANGELOG(s), comparison to other nRF24 libraries.
2. Showcase the API for various `rf24-rs` bindings.

This supplemental documentation is built with [mkdocs] but also requires
the bindings to be built and some other python dependencies installed.
In short, we have a [nur] task to quickly get the docs built locally for visual inspection.

```shell
nur docs
```

> [!TIP]
> Pass `--open` (`-o`) to automatically open the built docs in you default browser.
>
> Pass `--dirty` (`-d`) if you want to expedite the process for consecutive builds.

Under the hood, `nur docs` uses

- [yarn] to install javascript dependencies and build [typdedoc] output
- [uv] to install python dependencies and run [mkdocs].

[typedoc]: https://www.npmjs.com/package/typedoc
[mkdocs]: https://www.mkdocs.org/

### Rust API documentation

Documentation for the rust API is hosted at docs.rs automatically upon release.
To verify any documentation changes locally, we can use [nur] for that too:

```shell
nur docs rs
```

> [!TIP]
> Optional arguments are documented and shown in `nur docs rs -h`.
>
> Pass `--open` (or `-o`) to automatically open the built documentation in your default browser.

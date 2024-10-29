set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# run the test suite
[group("code coverage")]
test profile='default':
    cargo llvm-cov --no-report \
    nextest \
    --lib --tests --color always --profile {{ profile }}

# Clear previous test build artifacts
[group("code coverage")]
test-clean:
    cargo llvm-cov clean

# pass "--open" to this recipe's args to load HTML in your browser
# generate pretty coverage report
[group("code coverage")]
pretty-cov *args='':
    cargo llvm-cov report --json --output-path coverage.json
    llvm-cov-pretty coverage.json {{ args }}

# pass "--open" to this recipe's args to load HTML in your browser
# generate detailed coverage report
[group("code coverage")]
llvm-cov *args='':
    cargo llvm-cov report --html {{ args }}

# generate lcov.info
[group("code coverage")]
lcov:
    cargo llvm-cov report --lcov --output-path lcov.info

# pass "--open" to this recipe's "open" arg to load HTML in your browser
# serve mkdocs
[group("docs")]
docs open='':
    mkdocs serve --config-file docs/mkdocs.yml {{ open }}

# build mkdocs
[group("docs")]
docs-build:
    mkdocs build --config-file docs/mkdocs.yml

# pass "--open" to this recipe's "open" arg to load HTML in your browser
# rust API docs
[group("docs")]
docs-rs open='':
    cargo doc --no-deps --lib --manifest-path Cargo.toml {{ open }}

# run clippy and rustfmt (on library only)
lint:
    cargo clippy --allow-staged --allow-dirty --fix
    cargo fmt


# run clippy and rustfmt (on examples/rust only)
lint-examples:
    cargo clippy \
    --manifest-path examples/rust/Cargo.toml \
    --features linux \
    --allow-staged \
    --allow-dirty \
    --fix
    cargo fmt --manifest-path examples/rust/Cargo.toml

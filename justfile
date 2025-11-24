check:
    cargo clippy

build:
    cargo build

test *args:
    #!/usr/bin/env nu
    if (which cargo-nextest | is-empty) {
        cargo test {{args}}
    } else {
        cargo nextest run {{args}}
    }

bench:
    cargo bench --features bench --bench parsing

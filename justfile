
test:
    cargo test --workspace --all-features

check:
    cargo check --workspace --all-targets
    cargo check --workspace --all-targets --no-default-features

clippy:
    cargo clippy --fix --workspace --all-targets --all-features --allow-dirty --allow-staged -- -Dwarnings
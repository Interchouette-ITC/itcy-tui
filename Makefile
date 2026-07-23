.PHONY: lint test run build

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

# Quiet cargo so compile banners do not stick under the first TUI frame.
run:
	cargo run -q

build:
	cargo build --release

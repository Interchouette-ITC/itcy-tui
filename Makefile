.PHONY: lint test run build

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

run:
	cargo run

build:
	cargo build --release

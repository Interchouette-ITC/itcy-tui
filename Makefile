.PHONY: lint test run build clean check-license-headers apply-license-headers

lint: check-license-headers
	cargo fmt --check
	cargo clippy --all-targets -- \
		-D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery

check-license-headers:
	node scripts/check-license-headers.mjs

apply-license-headers:
	node scripts/apply-license-headers.mjs

test:
	cargo test

# One shell: build then exec so cargo status lines finish before the TUI frame.
run:
	@cargo build --release && exec ./target/release/itcy-tui

build:
	cargo build --release

clean:
	cargo clean

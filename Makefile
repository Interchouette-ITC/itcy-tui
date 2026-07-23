.PHONY: lint test run build

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

# One shell: build then exec binary so make never prints on top of the TUI frame.
run:
	@cargo build -q --release && exec ./target/release/itcy-tui

build:
	cargo build --release

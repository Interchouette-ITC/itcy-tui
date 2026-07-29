.PHONY: lint test run build check-license-headers apply-license-headers

lint: check-license-headers
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

check-license-headers:
	node scripts/check-license-headers.mjs

apply-license-headers:
	node scripts/apply-license-headers.mjs

test:
	cargo test

# One shell: build then exec binary so make never prints on top of the TUI frame.
run:
	@cargo build -q --release && exec ./target/release/itcy-tui

build:
	cargo build --release

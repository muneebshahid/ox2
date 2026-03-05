.PHONY: fmt lint run

fmt:
	cargo fmt

lint:
	cargo clippy -- -D clippy::pedantic -D clippy::nursery

run:
	cargo run

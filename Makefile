.PHONY: run test doc

run:
	cargo run

test:
	cargo test --lib

doc:
	cargo doc --no-deps --open

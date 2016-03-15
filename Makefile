.PHONY: run test doc

run:
	cargo run data/dmg.rom

test:
	cargo test --lib

doc:
	cargo doc --no-deps --open

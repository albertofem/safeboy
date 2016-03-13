.PHONY: run test doc

run:
	cargo run data/tetris.gb

test:
	cargo test --lib

doc:
	cargo doc --no-deps --open

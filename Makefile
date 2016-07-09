.PHONY: run test doc

run:
	RUST_BACKTRACE=1 cargo run data/tetris.gb

test:
	RUST_BACKTRACE=1 cargo test --lib

doc:
	cargo doc --no-deps --open

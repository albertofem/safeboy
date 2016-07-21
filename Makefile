.PHONY: run test doc

tetris:
	RUST_BACKTRACE=1 cargo run data/tetris.gb

zelda:
	RUST_BACKTRACE=1 cargo run data/zelda.gb

test:
	RUST_BACKTRACE=1 cargo test --lib

doc:
	cargo doc --no-deps --open

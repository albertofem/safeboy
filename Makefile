.PHONY: run test doc

tetris:
	RUST_BACKTRACE=1 cargo run -- --rom data/tetris.gb

zelda:
	RUST_BACKTRACE=1 cargo run -- --rom data/zelda.gb

test:
	RUST_BACKTRACE=1 cargo test --lib

doc:
	cargo doc --no-deps --open

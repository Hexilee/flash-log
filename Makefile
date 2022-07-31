fmt:
	cargo +nightly fmt
build:
	cargo build
release:
	cargo build --release
bench:
	cargo test --release -- --nocapture
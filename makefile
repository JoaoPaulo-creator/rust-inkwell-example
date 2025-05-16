build:
	cargo build -j 12
	./target/debug/toy_compiler input.toy

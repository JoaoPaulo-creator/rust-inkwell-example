build:
	cargo build -j 12
	./target/debug/toy_compiler input.toy > program.ll

llvm:
	llc -filetype=obj program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

release:
	cargo build --release -j 12

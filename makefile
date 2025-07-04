build:
	cargo build -j 12
	./target/debug/toy_compiler

all:
	cargo build -j 12
	./target/debug/toy_compiler input.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

algo:
	cargo build -j 12
	./target/debug/toy_compiler algo.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec


array:
	cargo build -j 12
	./target/debug/toy_compiler arrays.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

fct:
	cargo build -j 12
	./target/debug/toy_compiler functions.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec


while:
	./target/debug/toy_compiler arrays.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

llvm:
	llc -filetype=obj -relocation=no-pie program.ll -o program.o
	clang program.o -o toy
	./toy_exec

release:
	cargo build --release -j 12

clean:
	rm program.ll program.o toy

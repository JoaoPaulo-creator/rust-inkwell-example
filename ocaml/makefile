build:
	dune build
	./_build/default/bin/main.exe

all:
	dune build
	./_build/default/bin/main.exe
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

algo:
	dune build
	./_build/default/bin/main.exe algo.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec


array:
	dune build
	./_build/default/bin/main.exe arrays.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

fct:
	dune build
	./_build/default/bin/main.exe functions.toy
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

clean:
	dune clean

build:
	dune build

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
	./_build/default/bin/main.exe while.toy
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

llvm:
	llc -filetype=obj -relocation-model=pic program.ll -o program.o
	clang program.o -o toy_exec
	./toy_exec

clean:
	dune clean
	rm program.ll

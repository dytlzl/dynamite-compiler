.PHONY: test test-linux compile assemble run run-asm
test:
	cargo build
	./test/test.sh

test-linux:
	cargo build
	./test/test.sh -no-pie

src := ./temp/main.c

compile:
	cargo run $(src) > ./temp/main.s

assemble:
	cc -o ./temp/main ./temp/main.s

run: compile assemble
	./temp/main

run-asm: assemble
	./temp/main
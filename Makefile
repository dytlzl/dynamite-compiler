.PHONY: test test-linux compile assemble run run-asm
test:
	cargo build
	./test/test.sh $(if $(linux),-no-pie,)

test-linux:
	docker run --rm \
        -v $(shell pwd):/home \
        -w /home \
        -it rust:1.46 \
        make test linux=1

src := ./temp/main.c

compile:
	cargo run $(src) > ./temp/main.s

assemble:
	cc $(if $(linux),-no-pie,) -o ./temp/main ./temp/main.s 

run: compile assemble
	./temp/main

run-asm: assemble
	./temp/main
.PHONY: test test-linux compile assemble run run-asm
test:
	cargo build
	./test/test.sh $(if $(linux),-no-pie,)

test-linux-amd64:
	docker run --platform linux/amd64 --rm \
        -v $(shell pwd):/workspace \
        -w /workspace \
        -it rust:1.72.0 \
        make test linux=1

test-linux-arm64:
	docker run --platform linux/arm64 --rm \
        -v $(shell pwd):/workspace \
        -w /workspace \
        -it rust:1.72.0 \
        make test linux=1

src := ./test/expr.c

compile:
	cargo run -- $(if $(debug),--debug,) $(src) > ./temp/main.s

compile-linux:
	docker run --platform linux/amd64 --rm \
        -v $(shell pwd):/workspace \
        -w /workspace \
        -it rust:1.72.0 \
        make compile $(if $(debug),debug=1,)

assemble:
	cc $(if $(linux),-no-pie,) -o ./temp/main ./temp/main.s 

run: compile assemble
	./temp/main

run-asm: assemble
	./temp/main

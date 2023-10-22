docker_image := rust:1.72.0
test_options := --target x86_64-apple-darwin
src := ./tests/c/simple.c

ci:
	cargo fmt
	cargo clippy
	cargo test $(test_options)

test-linux:
	docker run --platform linux/amd64 --rm \
        -v $(shell pwd):/workspace \
        -w /workspace \
        -it $(docker_image) \
        cargo test

test-linux-aarch64:
	docker run --platform linux/arm64 --rm \
        -v $(shell pwd):/workspace \
        -w /workspace \
        -it $(docker_image)  \
        cargo test

create-temp:
	mkdir -p ./temp/binary              

c2b: create-temp
	cargo run $(src) > ./temp/temp.ll
	cc -o ./temp/binary/temp ./temp/temp.ll
	./temp/binary/temp

l2b: create-temp
	cc -o ./temp/binary/temp ./temp/temp.ll
	./temp/binary/temp

s2b: create-temp
	cc -o ./temp/binary/temp ./temp/temp.s
	./temp/binary/temp

ccc2s: create-temp
	cc -S -O0 -o ./temp/temp_cc.s $(src)

ccc2l: create-temp
	cc -S -emit-llvm -O0 -o ./temp/temp_cc.ll $(src)

ccs2b: create-temp
	cc -o ./temp/binary/temp_cc ./temp/temp_cc.s
	./temp/binary/temp_cc

ccc2b: ccc2s ccs2b

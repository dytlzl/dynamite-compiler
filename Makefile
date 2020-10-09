test:
	cargo build
	./test.sh

test-linux:
	cargo build
	./test.sh -no-pie

run:
	cargo run ./temp/main.c > ./temp/main.s && cc -o ./temp/main ./temp/main.s && ./temp/main
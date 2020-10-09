test:
	cargo build
	./test.sh

test-linux:
	cargo build
	./test.sh -no-pie

run:
	cargo run "$(source)" > ./asm/main.s && cc -o ./bin/main ./asm/main.s "$(link)" && ./bin/main
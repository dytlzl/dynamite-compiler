test:
	cargo build
	./test.sh

run:
	cargo run "$(source)" > ./asm/main.s && cc -o ./bin/main ./asm/main.s "$(link)" && ./bin/main
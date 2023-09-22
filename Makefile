docker_image := rust:1.72.0
test_options := --target x86_64-apple-darwin

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

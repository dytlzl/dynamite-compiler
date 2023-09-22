docker_image := rust:1.72.0

test:
	cargo test --target x86_64-apple-darwin

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
        make test linux=1

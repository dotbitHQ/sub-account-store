current_dir := $(shell pwd)

# The image nervos/ckb-docker-builder:bionic-rust-1.61.0 is require for building rocksdb crate, so do not change it.
# Building with `docker run` makes caching the cargo home possiable, so do not move it to the Dockerfile.
docker-build:
	docker run --rm -it \
		--network host \
		-v ${current_dir}/docker/.cargo:/usr/local/cargo/registry \
		-v ${current_dir}/docker/target:/app/target \
		-v ${current_dir}:/app \
		nervos/ckb-docker-builder:bionic-rust-1.61.0 \
		/bin/bash -c "cd /app && cargo build --release"
	mkdir ./build
	cp ./docker/target/release/rpc_server ./build/rpc_server

docker-image: docker-build
	docker build -t dotbitteam/sub-account-store:latest .

docker-test:
	docker run --rm -it --name sub-account-store \
		-p 9130:9130 \
		dotbitteam/sub-account-store:latest

docker-publish:
	docker image push dotbitteam/sub-account-store:latest

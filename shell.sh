docker build . --tag rust-shell
docker run -e "CARGO_INCREMENTAL=1" -it --privileged --rm -v "$(pwd):/src" --volumes-from cargo-cache rust-shell:latest bash
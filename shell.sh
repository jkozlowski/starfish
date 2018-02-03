docker build . --tag rust-shell
docker run -e "CARGO_INCREMENTAL=1" -it --rm -v "$(pwd):/src" --volumes-from cargo-cache rust-shell:latest bash
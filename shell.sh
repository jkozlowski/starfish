docker build . --tag rust-shell
docker run -it --rm -v $(pwd):/src --volumes-from cargo-cache rust-shell:latest bash
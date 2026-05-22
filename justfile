build:
    cargo build --workspace --exclude photopipeline-server

check:
    cargo check --workspace --exclude photopipeline-server

test:
    cargo test --workspace --exclude photopipeline-server

lint:
    cargo clippy --workspace --exclude photopipeline-server -- -D warnings

fmt:
    cargo fmt --all -- --check

clean:
    cargo clean

run-pipeline config input output:
    cargo run -- pipeline run -c {{config}} -i {{input}} -o {{output}}

run-server:
    cargo run -p photopipeline-server

# Contributing

Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, and `cargo doc --all-features --no-deps` before opening a change. Keep graph data independent of egui internals, document public APIs, and add a regression test for behavior changes. Do not include credentials or generated `target/` files.

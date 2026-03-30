#!/bin/bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p app-http -p http-middleware

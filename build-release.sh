#!/bin/sh

CC=clang
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none"
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features="optimize_for_size" --release --target=arm-unknown-linux-musleabihf

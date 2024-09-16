#!/bin/sh

CC=clang
cargo build --release --target=arm-unknown-linux-musleabihf

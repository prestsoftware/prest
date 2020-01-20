#!/bin/bash

TARGET="x86_64-pc-windows-gnu"

# rustup target add x86_64-pc-windows-msvc
cargo build --target="$TARGET" --release

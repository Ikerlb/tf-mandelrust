#!/bin/bash

target=x86_64-unknown-linux-musl

cd resources
#RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target $target 

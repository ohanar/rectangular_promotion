#!/usr/bin/bash

set -e

rustup update
git pull
cargo build --release
cp target/release/librectangular_promotion.so ../rectangular_promotion.so

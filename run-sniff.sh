#!/bin/bash
set -e

LD_LIBRARY_PATH=$(realpath ./sniff/lib/x86_64) \
DISCORD_GAME_SDK_PATH=$(realpath ./sniff) \
cargo run --manifest-path sniff/Cargo.toml

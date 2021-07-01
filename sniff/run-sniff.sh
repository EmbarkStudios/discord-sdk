#!/bin/bash
set -e

LD_LIBRARY_PATH=$(realpath ./sniff/lib/x86_64) cargo run --manifest-path "$(git rev-parse --show-toplevel)/sniff/Cargo.toml"

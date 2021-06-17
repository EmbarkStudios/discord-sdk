#!/bin/bash
set -e

LD_LIBRARY_PATH=$(realpath ./sniff/lib/x86_64) cargo run --manifest-path sniff/Cargo.toml

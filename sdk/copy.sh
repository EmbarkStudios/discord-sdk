#!/usr/bin/env bash

root=$(git rev-parse --show-toplevel)

cp "$root/sdk/README.md" "$root/sdk/CHANGELOG.md" "$root"

#!/usr/bin/env bash
set -ex

root=$(git rev-parse --show-toplevel)

cargo release --manifest-path "$root/sdk/Cargo.toml" --skip-push "$1"

tag=$(git tag --points-at HEAD)

cp "$root/sdk/README.md" "$root/sdk/CHANGELOG.md" "$root"

git add .
git commit --amend --no-edit
git tag -fa "$tag"
#git push

#!/usr/bin/env bash
set -e

root=$(git rev-parse --show-toplevel)

cargo release --execute --manifest-path "$root/sdk/Cargo.toml" --no-push "$1"

tag=$(git tag --points-at HEAD)

cp "$root/sdk/README.md" "$root/sdk/CHANGELOG.md" "$root"

git add .
git commit --amend --no-edit
git tag -fa -m "Release $tag" "$tag"
git push --follow-tags

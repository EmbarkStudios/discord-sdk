#!/bin/bash

set -e

index=${1:-0}

sniffed="$XDG_RUNTIME_DIR/discord-ipc-$index"
original="$XDG_RUNTIME_DIR/discord-ipc-$index.original"

mv "$sniffed" "$original"
socat -t100 -v "UNIX-LISTEN:$sniffed,mode=777,reuseaddr,fork" "UNIX-CONNECT:$original"

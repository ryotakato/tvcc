#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [ "$#" != "1" ]; then
    echo "Usage: $0 <path/to/program to debug>" >&2
    exit 1
fi

prog="$1"

# Start program in background
ROSETTA_DEBUGSERVER_PORT=1234 "$prog" &

# Run real gdb and tell it to attach
LC_ALL=C.UTF-8 /usr/bin/gdb \
    -iex "set architecture i386:x86-64" \
    -iex "file $prog" \
    -iex "target remote localhost:1234" \
    -iex "set history save on" \
    -iex "set disassembly-flavor intel" \
    -x "$SCRIPT_DIR/tvcc-ctx.py"

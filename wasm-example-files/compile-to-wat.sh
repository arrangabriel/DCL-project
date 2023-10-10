#! /bin/bash
# Compile a c file to .wat format
set -e

workdir=$(dirname "$(readlink -f "$0")")
cfile=$workdir/c/$1.c
wasmdir=$workdir/wasm
mkdir -p "$wasmdir"
wasmfile=$wasmdir/$1.wasm
watfile=$wasmdir/$1.wat

gcc \
   --target=wasm32 \
   -O1 \
   -flto \
   -nostdlib \
   -Wl,--no-entry \
   -Wl,--export-all \
   -o "$wasmfile" \
   "$cfile"

wasm-tools print "$wasmfile" > "$watfile"

if [[ $2 == "-v" ]]; then
    cat "$watfile"
fi

#! /bin/sh
# Compile a c file to .wat format
set -e

workdir=$(dirname $(readlink -f $0))
cfile=$workdir/c/$1.c
wasmdir=$workdir/wasm
mkdir -p $wasmdir
wasmfile=$wasmdir/$1.wasm
watfile=$wasmdir/$1.wat

clang \
   --target=wasm32 \
   -O3 \
   -flto \
   -nostdlib \
   -Wl,--no-entry \
   -Wl,--export-all \
   -Wl,--lto-O3 \
   -o $wasmfile \
   $cfile

wasm-tools print $wasmfile > $watfile

if [[ $2 == "-v" ]]; then
    cat $watfile
fi

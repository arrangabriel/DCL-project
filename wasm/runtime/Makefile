CC := ${WASI_SDK_PATH}/bin/clang
CFLAGS := -g -O3 -I./ -Wall

check-%-split: %.wat runtime-%.wat
	cargo run -q $< 6 | wasmtime run --preload contract=- $(word 2,$^)

check-%: %.wat runtime-%.wat
	wasmtime run --preload contract=$^

%.wat: %.wasm
	wasm2wat $< -o $@

runtime-%.wasm: runtime-%.c
	$(CC) $(CFLAGS) --target=wasm32-wasi -D__WASM__ \
              $< -o $@

%.wasm: %.c
	$(CC) $(CFLAGS) --target=wasm32 -D__WASM__ --no-standard-libraries \
              -Wl,--export-all -Wl,--no-entry $< -o $@

clean:
	-rm *.wasm *.wat *.elf

.NOTINTERMEDIATE:
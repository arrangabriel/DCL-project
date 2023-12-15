CC := ${WASI_SDK_PATH}/bin/clang
CFLAGS := -g -O3 -I./ -Wall

.check-profiler:
	@test $${PROFILER?Please set environment variable PROFILER}

check-%-split-profile: %-split.wat runtime-%-profile.wat .check-profiler
	${PROFILER} wasmtime run --preload contract=$^

check-%-profile: %.wat runtime-%-profile.wat .check-profiler
	${PROFILER} wasmtime run --preload contract=$^

check-%-split: %-split.wat runtime-%.wat
	wasmtime run --preload contract=$^

check-%: %.wat runtime-%.wat
	wasmtime run --preload contract=$^

analyze-%: %.wat
	cargo run -q analyze $< standard

%-split.wat: %.wat
	cargo run -q split $< 6 > $@

%.wat: %.wasm
	wasm2wat $< -o $@

runtime-%-profile.wasm: runtime-%.c
	$(CC) $(CFLAGS) -D PROFILE --target=wasm32-wasi -D__WASM__ \
              $< -o $@

runtime-%.wasm: runtime-%.c
	$(CC) $(CFLAGS) --target=wasm32-wasi -D__WASM__ \
              $< -o $@

%.wasm: %.c
	$(CC) $(CFLAGS) --target=wasm32 -D__WASM__ --no-standard-libraries \
              -Wl,--export-all -Wl,--no-entry $< -o $@

clean:
	-rm *.wasm *.wat *.elf
.NOTINTERMEDIATE:

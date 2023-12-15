CC := ${WASI_SDK_PATH}/bin/clang
CFLAGS := -g -O3 -I./ -Wall

.check-profiler:
	@test $${PROFILER?Please set environment variable PROFILER}

.check-state-size:
	@test $${STATE_SIZE?Please set environment variable STATE_SIZE}

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

%-split.wat: %.wat .check-state-size
	cargo run -q split $< ${STATE_SIZE} ${SPLIT_FLAGS} > $@

%.wat: %.wasm
	wasm2wat $< -o $@
	rm $^

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

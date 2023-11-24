WASI_SDK_PATH := /Users/arrankg/dev/university/DCLproject/wasi-sdk-20.0
CC := ${WASI_SDK_PATH}/bin/clang
CFLAGS := -g -O3 -I./
MAKEFLAGS += --no-print-directory

check:
	$(MAKE) check-no-abi-native
	@printf "\n\n"
	$(MAKE) check-no-abi-wasm
	@printf "\n\n"
	$(MAKE) check-abi-0-native
	@printf "\n\n"
	$(MAKE) check-abi-0-wasm


check-%-native: system-%.elf
	./$<

check-%-wasm: payment-%.wasm runtime-%.wasm
	wasmtime run --preload contract=$^

check-%: payment-%.wasm runtime-abi-0.wasm
	wasmtime run --preload contract=$^

system-%.elf: payment-%.c runtime-%.c
	$(CC) $(CFLAGS) $^ -o $@

payment-%.wasm: payment-%.c
	$(CC) $(CFLAGS) --target=wasm32 -D__WASM__ --no-standard-libraries \
              -Wl,--export-all -Wl,--no-entry $< -o $@

runtime-%.wasm: runtime-%.c
	$(CC) $(CFLAGS) --target=wasm32-wasi -D__WASM__ \
              $< -o $@

%.wat: %.wasm
	wasm2wat $< -o $@

clean:
	-rm *.wasm *.wat *.elf

.NOTINTERMEDIATE:
.PHONY: check check-%

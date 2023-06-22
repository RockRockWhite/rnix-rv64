BUILD:=./target
SRC:=./src
TARGET := riscv64gc-unknown-none-elf
MODE := release

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

$(BUILD)/$(TARGET)/$(MODE)/os: $(SRC)/*
	cargo build $(MODE_ARG)

$(BUILD)/$(TARGET)/$(MODE)/os.bin: $(BUILD)/$(TARGET)/$(MODE)/os
	$(OBJCOPY) $< --strip-all -O binary $@

test: $(BUILD)/$(TARGET)/$(MODE)/os.bin

.PHONY: clean
clean:
	cargo clean

.PHONY: qemu
qemu: $(BUILD)/$(TARGET)/$(MODE)/os.bin
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=$<,addr=0x80200000

.PHONY: qemug
qemug: $(BUILD)/$(TARGET)/$(MODE)/os.bin
	qemu-system-riscv64 \
	-S -s \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=$<,addr=0x80200000
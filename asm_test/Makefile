main.elf : main.s
	riscv64-elf-gcc -nostdlib -nostdinc -static -g -Ttext 0x80200000 main.s -o main.elf

.PHONY: clean
clean:
	rm *.elf

.PHONY: qemug
qemug: main.elf
	qemu-system-riscv64 \
	-m 2G \
	-machine virt \
	-S -s \
	-nographic \
	-bios none \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=$<,addr=0x80200000
	# --kernel $<
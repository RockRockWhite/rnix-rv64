main.elf : main.s
	riscv64-elf-gcc -nostdlib -nostdinc -static -g -Ttext 0x80000000 main.s -o main.elf

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
	--kernel $<
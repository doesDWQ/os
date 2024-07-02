


all:
    # 编译内核
	cargo build

    # 去除elf中的头信息
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/debug/os -O binary target/riscv64gc-unknown-none-elf/debug/os.bin

    # qemu启动内核
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/debug/os.bin,addr=0x80200000 \
    -s -S

.PHONY: all

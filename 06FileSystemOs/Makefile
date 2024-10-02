.PHONY: build run

build:
	cd ../06FileSystemUser && cargo build --release
	cd ../easy-fs-fuse && cargo run --release -- -s ../06FileSystemUser/src/bin/ -t ../06FileSystemUser/target/riscv64gc-unknown-none-elf/release/

	# 编译内核
	cd ../06FileSystemOs && cargo build --release

    # 去除elf中的头信息
	# rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin

run: build
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os,addr=0x80200000 \
	-drive file=../06FileSystemUser/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -s -S

gdbclient:
	riscv64-unknown-elf-gdb  -ex 'file target/riscv64gc-unknown-none-elf/release/os' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'


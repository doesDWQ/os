.PHONY : fs

fs:
	cd ../06FileSystemUser && make build
	cd ../easy-fs-fuse && cargo run --release -- -s ../06FileSystemUser/src/bin/ -t ../06FileSystemUser/target/riscv64gc-unknown-none-elf/release/
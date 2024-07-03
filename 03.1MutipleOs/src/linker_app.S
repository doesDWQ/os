# os/src/link_app.S

    .align 3    # 对齐到2的3次方，8字节上
    .section .data
    .global _num_app    # 定义_num_app全局数据段
_num_app:
    .quad 5             # .quad 伪指令，定义一个64位的整数，_num_app相当于连续存储了7个64位的整数
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_4_end

    .section .data
    .global app_0_start
    .global app_0_end
app_0_start:
    .incbin "../03.1MutipleUser/target/riscv64gc-unknown-none-elf/release/00hello_world.bin" # .incbin指令，将文件处的内容插入到目标中
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
app_1_start:
    .incbin "../03.1MutipleUser/target/riscv64gc-unknown-none-elf/release/01store_fault.bin"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
app_2_start:
    .incbin "../03.1MutipleUser/target/riscv64gc-unknown-none-elf/release/02power.bin"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
app_3_start:
    .incbin "../03.1MutipleUser/target/riscv64gc-unknown-none-elf/release/03priv_inst.bin"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
app_4_start:
    .incbin "../03.1MutipleUser/target/riscv64gc-unknown-none-elf/release/04priv_csr.bin"
app_4_end:
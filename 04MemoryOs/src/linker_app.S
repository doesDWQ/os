# os/src/link_app.S

    .align 3    # 对齐到2的3次方，8字节上
    .section .data
    .global _num_app    # 定义_num_app全局数据段
_num_app:
    .quad 3             # .quad 伪指令，定义一个64位的整数，_num_app相当于连续存储了7个64位的整数
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_2_end

    .section .data
    .global app_0_start
    .global app_0_end
app_0_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/00write_a" # .incbin指令，将文件处的内容插入到目标中
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
app_1_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/01write_b"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
app_2_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/02write_c"
app_2_end:
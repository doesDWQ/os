
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 7
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_6_end

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/00power_3"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/01power_5"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/02power_7"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/03sleep"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/04load_fault"
app_4_end:

    .section .data
    .global app_5_start
    .global app_5_end
    .align 3
app_5_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/05store_fault"
app_5_end:

    .section .data
    .global app_6_start
    .global app_6_end
    .align 3
app_6_start:
    .incbin "../04MemoryUser/target/riscv64gc-unknown-none-elf/release/sbrk_test"
app_6_end:

.altmacro
.macro SAVE_GP n     # 定义宏汇编，名字是SAVE_GP，参数是n
    sd x\n, \n*8(sp)    # 展开将X1到Xn寄存器存储到栈指针sp 偏移量位n*8的位置，这里假设每个寄存器占据8字节，即64位
.endm                # 表示汇编宏结束
.macro LOAD_GP n
    ld x\n, \n*8(sp)    # sd 保存到内存中，ld加载到内存中
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2            # 将下面的__alltraps的地址4字节对齐，这是risc-v特权级规范的要求
__alltraps:
    csrrw sp, sscratch, sp  # 交换 sscratch， sp 的值
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*8      # 栈的位置移动34个 8字节
    # save general-purpose registers
    sd x1, 1*8(sp)          # x1寄存器写入栈中第一个8字节
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)          # x3寄存器写入栈中第3个8字节
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5               # 将x5到下31 27个寄存器保存到栈对应的位置
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus    # 将sstaus存入t0
    csrr t1, sepc       # 将sepc存入t1
    sd t0, 32*8(sp)     # 将t0,存入第32个8字节
    sd t1, 33*8(sp)     # 将t1,存入第33个8字节
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch   # 将 sscratch 存入第t2寄存器
    sd t2, 2*8(sp)      # 将 t2 写入第二个8字节，因为sp寄存器也是t2寄存器
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp           # 将sp寄存器赋值给a0
    call trap_handler   # 调用陷入处理函数

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    # mv sp, a0       # 将sp 指向a0, 在多任务切换时，换栈的操作发生在任务切换的时候
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp) # 还原t0
    ld t1, 33*8(sp) # 还原t1
    ld t2, 2*8(sp)  # 还原t2
    csrw sstatus, t0 # 还原status
    csrw sepc, t1       # 还原sepc
    csrw sscratch, t2   # 还原sscratch
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)      # 还原x1, x2是sp栈寄存器，不需要保存
    ld x3, 3*8(sp)      # 还原 x3， x4是线程相关寄存器
    .set n, 5           # 还原 x5 到 31
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8   
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp  # 交换用户和内核栈指针
    sret     # 运行结束，返回u特权级别继续执行

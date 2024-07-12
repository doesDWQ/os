


#![no_std] // 去除掉标准库依赖
#![no_main] // 去除main主函数
#![feature(panic_info_message)] // panic_handler中打印info使用
#![feature(alloc_error_handler)]
#[allow(warnings)]


mod lang_items;
mod sbi;
#[macro_use]
mod console;   
mod logging;
mod loader;
mod sync;
mod syscall;
mod trap;
mod task;
mod timer;
mod config;
mod mm;

#[path = "boards/qemu.rs"]
mod board;

// 引入内存分配
extern crate alloc;

use log::*;
use mm::remap_test;

#[macro_use]
extern crate bitflags;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("linker_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    // 清空bss段信息
    clear_bss();
    // 打印段信息
    // print_code_segment();

    // 初始化内存
    mm::init();
    // 测试内存
    mm::remap_test();
    // 初始化中断
    trap::init();
    // 打开时间中断
    trap::enable_timer_interrupt();
    // 设置时间中断触发
    timer::set_next_trigger();
    // 运行第一个任务
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

// 打印代码段信息
fn print_code_segment() {
    logging::init();

    extern "C" {
        fn stext();     // 代码段开始地址
        fn etext();     // 代码段结束地址
        fn srodata();   // 只读数据开始地址
        fn erodata();   // 只读数据结束地址
        fn sdata();     // 可变数据开始地址
        fn edata();     // 可变数据结束地址
        fn sbss();      // 存储未初始化的变量开始地址
        fn ebss();      // 存储未初始化的变量结束地址
        fn boot_stack_lower_bound(); // 栈尾地址
        fn boot_stack_top();         // 栈头地址
    }

    trace!("[kernel]\t.text\t[{:#x},{:#x}]",    stext as usize, etext as usize);
    debug!("[kernel]\t.rodata\t[{:#x},{:#x}]",  srodata as usize, erodata as usize);
    info!("[kernel]\t.data\t[{:#x},{:#x}]",  sdata as usize, edata as usize);
    warn!("[kernel]\t.sbss\t[{:#x},{:#x}]",  sbss as usize, ebss as usize);
    error!("[kernel]\t.stack\t[{:#x},{:#x}]",  boot_stack_lower_bound as usize, boot_stack_top as usize);
}

// 清空bss数据段
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0)
        }
    })
}
// 该文件已人工核对过

#![deny(warnings)]
#![no_std] // 去除掉标准库依赖
#![no_main] // 去除main主函数
#![feature(panic_info_message)] // panic_handler中打印info使用
#![feature(alloc_error_handler)]


extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;


use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("linker_app.S"));


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

#[no_mangle]
pub fn rust_main() -> ! {
    // 清空bss段信息
    clear_bss();
    print!("[kernel] Hello, world!");
    // 初始化内存
    mm::init();
    println!("[kernel] back to world!");
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


#![no_std]
#![feature(linkage)]    // 符号链接属性？
#![feature(panic_info_message)]


#[macro_use]
pub mod console;

mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() ->! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit");
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

#[linkage = "weak"] // 表示弱链接
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

// 清空bss数据段
fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }

    (start_bss as usize..end_bss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0)
        }
    })
}

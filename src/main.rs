// os/src/main.rs
#![feature(panic_info_message)]
#![no_main]
#![no_std]
mod lang_items;
mod sbi;
mod console;


use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() ->! {
    clear_bss();
    println!("\x1b[31mhello world\x1b[0m");
    println!("\x1b[31mhello world1\x1b[0m");
    println!("\x1b[31mhello world3\x1b[0m");
    // panic!("Shutdown machine!");
    let mut start = 1;

    loop {
        println!("start: {}", start);
        start += 1;
        if start > 100 {
            break;
        }
    }

    loop {}
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
#![no_std]
#![no_main]

use core::arch::asm;

use user_lib::println;

extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Try to execute privileged instruction in U Mod");

    unsafe {
        asm!("sret");
    }

    0
}
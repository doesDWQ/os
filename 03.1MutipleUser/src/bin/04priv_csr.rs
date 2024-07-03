#![no_std]
#![no_main]

use core::arch::asm;

use riscv::register::sstatus::{self, SPP};
use user_lib::println;

extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Try to access priviledged CSR in U Mode");

    unsafe {
        sstatus::set_spp(SPP::User);
    }

    0
}
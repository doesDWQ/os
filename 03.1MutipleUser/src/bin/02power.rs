#![no_std]
#![no_main]

use user_lib::println;



#[macro_export]
extern crate user_lib;

const SIZE: usize = 10;
const P: u32 = 3;
const STEP: usize = 100000;
const MOD: u32 = 10007;

#[no_mangle]
fn main() -> i32 {
    let mut pow = [0u32; SIZE];
    let mut index: usize = 0;
    pow[index] = 1;

    for i in 1..=STEP {
        let last = pow[index];
        index = (index+1) % SIZE;
        pow[index] = last * P % MOD;
        if i % 10000 == 0 {
            println!("{}^{} = {} (Mod {})", P, i, pow[index], MOD);
        }
    } 

    println!("Test power OK !");
    0
}
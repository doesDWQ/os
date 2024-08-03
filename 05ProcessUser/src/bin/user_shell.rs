
#![no_std]
#![no_main]

use alloc::string::String;
use user_lib::{console::getchar, exec, fork, waitpid};

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;  // 换行按键 相当于 \r
const CR: u8 = 0x0du8;  // 回车按键 相当于 \n
const DL: u8 = 0x7fu8;  // delete 按键
const BS: u8 = 0x08u8;  // backspace 按键



#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">> ");

    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");

                if !line.is_empty() {
                    line.push('\0');
                    let pid = fork();

                    if pid == 0 {
                        // 子进程
                        if exec(line.as_str()) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", 
                            pid, exit_code);
                    }
                    line.clear();
                }
                print!(">> ");
            }

            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }

            _ => {
                print!("{}", c as char);
            }
        }
    }
    0
}
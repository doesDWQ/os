


mod fs;
mod process;

const Syscall_write: usize = 64;
const Syscall_exit: usize = 93;

use fs::*;
use process::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        Syscall_write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall_exit => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;


pub fn sys_exit(exit_code: i32) ->! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unableable in sys_exit!")
}

pub fn sys_yeield() -> isize {
    suspend_current_and_run_next(); 
    0
}

/// get time in milliseconds
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
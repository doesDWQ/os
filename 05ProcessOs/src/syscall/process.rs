// 该文件已人工核对过


use alloc::sync::Arc;

use crate::loader::get_app_data_by_name;
use crate::mm::{translated_refmut, translated_str};
use crate::timer::get_time_ms;

use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next
};

// use super::{syscall, SYSCALL_EXEC, SYSCALL_GETPID, SYSCALL_WAITPID};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get current time
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_fork() -> isize {
    // 获取当前任务
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;

    // 获取跳板上下文
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    trap_cx.x[10] = 0;

    // 添加新的任务上下文
    add_task(new_task);

    // 返回pid
    new_pid as isize
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();

    let mut inner = task.inner_exclusive_access();
    if inner.children.iter().find(|p| {
        pid == -1 || pid as usize == p.getpid()
    }).is_none() {
        return -1;
    }

    let pair = inner.children.iter().enumerate()
    .find(|(_, p)| {
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid() )
    });

    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();

        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) =  exit_code;

        found_pid as isize
    } else {
        -2
    }
    
}


pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) =  get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    }  else {
        -1
    }
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}
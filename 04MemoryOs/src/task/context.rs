// 该文件已人工核对过

use crate::trap::trap_return;

#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12], // s0-s11，属于被调用者保存寄存器，其他要么属于调用者保存，要么属于临时寄存器
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self{
            ra: 0,
            sp: 0,
            s:  [0; 12],
        }
    }

    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}


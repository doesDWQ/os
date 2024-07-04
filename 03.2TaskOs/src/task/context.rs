

#[derive(Copy, Clone)]
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
            s:  [0;12],
        }
    }

    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}


// 该文件已人工核对过


core::arch::global_asm!(include_str!("switch.S"));
use super::context::TaskContext;

extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut TaskContext,next_task_cx_ptr: *const TaskContext);
}
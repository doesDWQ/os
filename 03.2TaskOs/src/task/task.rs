

use crate::{loader::Max_app_num, sync::UPSafeCell};

use super::context::TaskContext;


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,     // 未初始化
    Ready,      // 准备运行
    Runding,    // 正在运行
    Exited,     // 已退出
}

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
}



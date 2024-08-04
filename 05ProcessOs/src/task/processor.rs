use alloc::sync::Arc;
use lazy_static::*;

use crate::{sync::UPSafeCell, trap::TrapContext};

use super::{manager::fetch_task, switch::__switch, task::{TaskControlBlock, TaskStatus}, TaskContext};

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>, // 当前进程
    idle_task_cx: TaskContext, // 当前任务上下文
}

impl Processor {
    // 新建一个进程对象
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }


    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut  self.idle_task_cx as *mut _
    }

   
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    // 获取当前任务上下文
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> =  unsafe {
        UPSafeCell::new(Processor::new())
    };
}


pub fn run_tasks() {
    loop {
        // 获取进程对象
        let mut processor = PROCESSOR.exclusive_access();

        // 获取第一个任务
        if let Some(task) = fetch_task() {
            // 获取任务上下文
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            
            // 获取任务
            let mut task_inner = task.inner_exclusive_access();
            // 获取切换任务的上下文
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            // 任务状态设置为进行中
            task_inner.task_status = TaskStatus::Runding;

            // 删除任务引用
            drop(task_inner);

            // 设置当前任务
            processor.current = Some(task);

            drop(processor);

            unsafe {
                __switch(
                    idle_task_cx_ptr, 
                    next_task_cx_ptr)
            }
        }

       
    }
}


pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor: core::cell::RefMut<Processor> = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();

    drop(processor);

    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr)
    }
}


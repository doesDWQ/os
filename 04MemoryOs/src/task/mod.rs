
mod context;
mod switch;
mod task;

use alloc::vec::Vec;
use context::TaskContext;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

// fn mark_current_suspended() {
//     TASK_MANAGER.
// }

// 任务管理器
pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];

        task0.task_status = TaskStatus::Runding;
        let mut _unsed = TaskContext::zero_init();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);

        unsafe {
            __switch(&mut _unsed as *mut TaskContext, next_task_cx_ptr);
        }

        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;

        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1).map(|id| id % self.num_app)
        .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_nex_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Runding;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);

            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            println!("All application completed!");
            shutdown(false);
        }
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

use crate::{loader::{get_app_data, get_num_app}, sbi::shutdown, sync::UPSafeCell, trap::TrapContext};

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init Task_manager");
        let num_app = get_num_app();
        println!("num_app={}", num_app);

        let mut tasks: Vec<TaskControlBlock> = Vec::new();

        for i in 0..num_app{
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ))
        }

        TaskManager{
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    current_task:0,
                })
            }
        }
    };
}


pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}


pub fn run_first_task(){
    TASK_MANAGER.run_first_task();
}

fn run_nex_task() {
    TASK_MANAGER.run_nex_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended()
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_nex_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_nex_task();
}
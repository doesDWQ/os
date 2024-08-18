
use super::TaskContext;
use super::pid::{pid_alloc, KernelStack,PidHandle};
use crate::config::TRAP_CONTEXT;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{MemorySet,PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,

    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}


impl TaskControlBlock {

    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    
    pub fn new(elf_data: &[u8]) -> Self {
        // 获取一个内存集合，用户栈，程序执行入口点
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // 获取跳板上下文
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap().ppn();

        // 获取进程对象，里面仅包含进程对象
        let pid_handle = pid_alloc();

        // 获取内核栈
        let kernel_stack = KernelStack::new(&pid_handle);
        // 获取内核栈顶
        let kernel_stack_top = kernel_stack.get_top();

        // 获取任务控制块
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                let value = TaskControlBlockInner{
                        trap_cx_ppn,
                        base_size: user_sp,
                        task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                        task_status: TaskStatus::Ready,
                        memory_set,
                        parent: None,
                        children: Vec::new(),
                        exit_code: 0,
                        fd_table: vec![
                            Some(Arc::new(Stdin)),
                            Some(Arc::new(Stdout)),
                            Some(Arc::new(Stdout))
                        ]
                    };
                UPSafeCell::new(
                    value
                )
            },
        };

        // 获取跳板上下文
        let trap_cx = task_control_block
            .inner_exclusive_access().get_trap_cx();

        // 修改跳板上文内容
        *trap_cx = TrapContext::app_init_context(
            entry_point, 
            user_sp, 
            KERNEL_SPACE.exclusive_access().token(), 
            kernel_stack_top, 
            trap_handler as usize);
        
        // 返回任务控制块
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        // 获取新进程配置
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // 获取跳板上线文
        let trap_cx_ppn = memory_set.translate(
            VirtAddr::from(TRAP_CONTEXT).into()
        ).unwrap().ppn();

        let mut inner = self.inner_exclusive_access();

        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;

        let trap_cx = inner.get_trap_cx();

        // 设置跳板上下文
        *trap_cx = TrapContext::app_init_context(
            entry_point, 
            user_sp, 
            KERNEL_SPACE.exclusive_access().token(), 
            self.kernel_stack.get_top(),
            trap_handler as usize);
    }

    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> { 
        let mut parent_inner = self.inner_exclusive_access();
        // 从用户任务内存中中复制内容
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);

        // 跳板上下文
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();

        // 获取pid对象
        let pid_handle = pid_alloc();
        // 获取pid对应的内核栈
        let kernel_stack = KernelStack::new(&pid_handle);

        // 获取栈顶
        let kernel_stack_top = kernel_stack.get_top();
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()))
            } else {
                new_fd_table.push(None)
            }
        }
       
        let task_control_bloc = Arc::new(
            TaskControlBlock{
                pid: pid_handle,
                kernel_stack, 
                inner: unsafe {
                    UPSafeCell::new(TaskControlBlockInner{
                        trap_cx_ppn,
                        base_size: parent_inner.base_size,
                        task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                        task_status: TaskStatus::Ready,
                        memory_set,
                        parent: Some(Arc::downgrade(self)),
                        children:Vec::new(),
                        exit_code:0,
                        fd_table: new_fd_table,
                    })
                }
        });

        // 加入新创建的子进程
        parent_inner.children.push(task_control_bloc.clone());
        
        let trap_cx = task_control_bloc.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;

        // 返回任务控制块
        task_control_bloc
    }

    
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

}


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,      // 准备运行
    Running,    // 正在运行
    Zombie,     
}

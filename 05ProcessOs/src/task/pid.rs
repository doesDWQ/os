use alloc::vec::Vec;



pub struct PidHandle(pub usize);

pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

// 进程id分配器
impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    
    // 申请一个进程id
    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            self.current += 1;
            PidHandle(self.current -1)
        }
    }

    // 回收一个进程id
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
            "pid {} has been deallocated", pid
        );
        self.recycled.push(pid);
    }
}

use lazy_static::*;

use crate::{config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE}, mm::{MapPermission, VirtAddr, KERNEL_SPACE}, sync::UPSafeCell};

lazy_static! {
    static ref PID_ALLOCaTOR: UPSafeCell<PidAllocator> = unsafe {
        UPSafeCell::new(PidAllocator::new())
    };
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCaTOR.exclusive_access().alloc()
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCaTOR.exclusive_access().dealloc(self.0);
    }
}

pub struct KernelStack {
    pid: usize,
}

// 获取指定app_id的内核栈上下限
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top-KERNEL_STACK_SIZE;
    (bottom, top)
}

impl KernelStack {
    // 获取内核栈对象
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);

        // 将内核栈插入内核空间
        KERNEL_SPACE.exclusive_access()
        .insert_framed_area(
            kernel_stack_bottom.into(), 
            kernel_stack_top.into(), 
            MapPermission::R | MapPermission::W,
        );

        // 返回内核栈对象
        KernelStack {
            pid: pid_handle.0,
        }
    }

    pub fn push_on_top<T>(&self, value: T) -> *mut T where T:Sized, {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe { *ptr_mut = value; }
        ptr_mut
    }

    // 获取内核栈top值
    pub fn get_top(&self) -> usize {
        let(_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();

        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}
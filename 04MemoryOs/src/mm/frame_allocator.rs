// 该文件已人工核对过

use super::address::{PhysAddr, PhysPageNum};
use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

// 动态物理帧跟踪器
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    // 获取一个帧
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self{ ppn }
    }
}

// 实现debug切片
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

// 实现Drop切片
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// 动态帧分配器接口
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

// 实际实现的栈帧分配器
pub struct StackFrameAllocator {
    current: usize, // 当前分配最大帧号
    end: usize,     // 当前能分配的最大帧号
    recycled: Vec<usize>, // 回收后的帧号
}

impl StackFrameAllocator {
    // 初始化帧分配器
    pub fn init (&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

// 实现动态帧分配器接口
impl FrameAllocator for StackFrameAllocator {
    // new新建一个分配器
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    
    // 申请一个物理页帧
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            // 如果能从回收中获取到帧则从回收中获取帧
            Some(ppn.into())
        } else {
            // 否则获取未分配过的帧
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current-1).into())
            }
        }
    }
    
    // 释放物理页帧
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled.iter().any(|&v| v==ppn) {
            panic!("Frame ppn = {:#x} has not been allocated!", ppn);
        }
        // 回收物理页号
        self.recycled.push(ppn);
    }
}

// 定义真分配器别名，方便替换实现
type FrameAllocatorImpl = StackFrameAllocator;

// 全局静态懒加载初始化帧分配器变量
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

// 初始化帧分配器
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    // ekernel表示分配器开始位置，MEMORY_END表示结束位置
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor());
}

// 对外提供分配接口
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

// 对外提供释放接口
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

// 帧分配器测试用例
#[allow(unused)]
pub fn frame_allocator_test(){
    let mut v: Vec<FrameTracker> = Vec::new();

    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }

    drop(v);
    println!("frame_allocator_test passed!");
}
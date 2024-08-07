// 该文件已人工核对过

mod address;
mod heap_allocator;
mod frame_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, PageTableEntry};
use page_table::{PTEFlags, PageTable};

pub fn init() {
    // 初始化堆空间
    heap_allocator::init_heap();
    // heap_allocator::heap_test();
    // 初始化帧分配器
    frame_allocator::init_frame_allocator();
    // frame_allocator::frame_allocator_test();
    // 启动sv39分页模式
    KERNEL_SPACE.exclusive_access().activate();
}

// 该文件已人工核对过

use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

// 内存管理器
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// 内存分配失败的情况
#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) ->! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

// 申请连续可变内存空间
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];


// 初始化堆空间
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.lock().init(
            HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE
        );
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    extern "C" {
        fn sbss();
        fn ebss();
    }

    let bss_range = (sbss as usize)..(ebss as usize);

    let a = Box::new(5);

    assert_eq!(*a, 5);

    let ptr = &(a.as_ref() as *const _ as usize);
    assert!(bss_range.contains(ptr));
    drop(a);

    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }

    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }

    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
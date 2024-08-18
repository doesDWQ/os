use easy_fs::BlockDevice;
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};
use lazy_static::*;
use alloc::vec::Vec;

use crate::{mm::{frame_alloc, StepByOne, frame_dealloc, kernel_token, FrameTracker, PageTable, PhysAddr, PhysPageNum, VirtAddr}, sync::UPSafeCell};


const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, VirtioHal>>);

pub struct VirtioHal;

impl VirtIOBlock {
    pub fn new() -> Self {
        unsafe{
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        }
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0.exclusive_access().read_block(block_id, buf).expect("Error when reading VirtIOBlk");
    }
    
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0.exclusive_access().write_block(block_id, buf).expect("Error when writing VirIOBlk");
    }
}

/*
    VirtIO 设备需要占用部分内存作为一个公共区域从而更好的和 CPU 进行合作。这就像 MMU 需要在内存中保存多级页表才能和 CPU 共同实现分页机制一样。在 VirtIO 架构下，需要在公共区域中放置一种叫做 VirtQueue 的环形队列，CPU 可以向此环形队列中向 VirtIO 设备提交请求，也可以从队列中取得请求的结果，详情可以参考 virtio 文档 。对于 VirtQueue 的使用涉及到物理内存的分配和回收，但这并不在 VirtIO 驱动 virtio-drivers 的职责范围之内，因此它声明了数个相关的接口，需要库的使用者自己来实现：
*/

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> virtio_drivers::PhysAddr {
        let mut ppn_base = PhysPageNum(0);
        for i in 0..pages {
            let frame: FrameTracker = frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }

            assert_eq!(frame.ppn.0, ppn_base.0 + i);

            QUEUE_FRAMES.exclusive_access().push(frame);
        }

        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    fn dma_dealloc(pa: virtio_drivers::PhysAddr, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: virtio_drivers::PhysAddr) -> virtio_drivers::VirtAddr {
        addr
    }

    fn virt_to_phys(vaddr: virtio_drivers::VirtAddr) -> virtio_drivers::PhysAddr {
        PageTable::from_token(kernel_token()).translate_va(VirtAddr::from(vaddr)).unwrap().0
    }
}



// 该文件已人工核对过

use super::{frame_alloc, FrameTracker, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;


bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;   // 当该位为1时，页表项才是合法的
        const R = 1 << 1;   // 控制索引到这个页表项的对应的虚拟页面是否允许读
        const W = 1 << 2;   // 控制索引到这个页表项的对应的虚拟页面是否允许写
        const X = 1 << 3;   // 控制索引到这个页表项的对应的虚拟页面是否允许执行
        const U = 1 << 4;   // 控制索引到这个页表项的对应的虚拟页面是否处于U特权级情况下是否被允许访问
        const G = 1 << 5;   // 暂时不理会
        const A = 1 << 6;   // 处理器自动更新，表示该页表项对应的虚拟页面是否被访问过
        const D = 1 << 7;   // 处理器自动更新，表示该页表项对应的虚拟页面是否被修改过
    }
}

#[derive(Copy, Clone)] // 使用copy， clone 两个trait就会保证在传递参数时所有权不会转移
#[repr(C)]      // 这个蛮重要的，以c的方式存储机结构体对象
pub struct PageTableEntry {
    pub bits: usize
}

impl PageTableEntry {
    // 通过物理页号新建立一个页表项
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    // 新建一个空的页表项
    pub fn empty() -> Self {
        PageTableEntry {
            bits:0,
        }
    }

    // 通过页表项获物理页面号，因为物理地址支持56位，并且最低11位等于实际偏移，
    // 因此 ((1usize << 44) -1 ) 就可以去取出物理页面号
    pub fn ppn(&self) -> PhysPageNum {
        // into 调用对于的From类型
        (self.bits >> 10 & ((1usize << 44) -1 )).into()
    }

    // 获取到flags
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    // 标志位判断
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn writeable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn executeable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}


pub struct PageTable {
    root_ppn: PhysPageNum,  // 根物理号
    frames: Vec<FrameTracker>, // 保存所有的物理帧
}

// 实际上就是一颗树
impl PageTable {
    // 建立一个pagetable
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();

        PageTable{
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    // 通过satp临时传递一个用来手动页表的pageTable
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) -1)),
            frames: Vec::new(),
        }
    }

    // 通过vpn搜索到实际控制的最底层的4k内存节点，不存在就创建
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }

        result
    }

    // 通过vpn搜索到实际控制的最底层的4k内存节点
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }

            ppn = pte.ppn();
        }

        result
    }

    // 最底层的结点指针和实际的物理内存页关联上
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped bofore mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    
    // 最底层的结点指针和实际的物理内存页释放掉关系
    #[allow(unused)]
    pub fn unmap(&mut self, vpn:VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invaild before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    
    // 通过vpn查询到pageTableTntry
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    // 虚拟地址转换为物理地址
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();

            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }

    // 安装satp csr要求构造 64 位无符号整数，使得其分页模式为 SV39 
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

// 通过页表获取到管辖的byte列表
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    // 获取到虚拟页表
    let page_table = PageTable::from_token(token);
    // 虚拟开始地址
    let mut start = ptr as usize;
    // 虚拟结束地址
    let end = start + len;
    let mut v = Vec::new();

    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));

        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }

        start = end_va.into();
    }
    v
}

// 根据地址从页表中读取到一个字符串
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut());

        // 字符串是以0结尾的
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }

    string
}

// 根据地址获取指向实际地址的可变指针
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;

    page_table
    .translate_va(VirtAddr::from(va)).unwrap()
    .get_mut()
}
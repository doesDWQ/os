// 该文件已人工核对过

use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{MEMORY_END, MMIO, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::sync::UPSafeCell;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::*;
use riscv::register::satp;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = 
        Arc::new(unsafe {
            UPSafeCell::new(MemorySet::new_kernel())
        });
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}


impl MemorySet {
    // new 一个空的memorySet
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    // 插入一块指定虚拟范围的逻辑段空间
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr, 
        end_va: VirtAddr,
        permission: MapPermission
    ){
        self.push(MapArea::new(start_va,end_va,MapType::Framed, permission), None);
    }

    // 加入一块新的逻辑段空间，并copy进入数据
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }

    // 绑定跳板位置
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(), 
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        )
    }

    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();

        // 设置跳板
        memory_set.map_trampoline();

        println!(".text {:#x}, {:#x}", stext as usize, etext as usize);
        println!(".rodata {:#x}, {:#x}", srodata as usize, erodata as usize);
        println!(".data {:#x}, {:#x}", sdata as usize, edata as usize);
        println!(".bss {:#x}, {:#x}", 
            sbss_with_stack as usize, ebss as usize);
        println!("mapping .text section");

        // 直接映射各个区域空间
        memory_set.push(MapArea::new(
           ( stext as usize).into(),
           (etext as usize).into(),
           MapType::Identical,
           MapPermission::R | MapPermission::X,
        ), None);

        println!("mapping .rodata section");
        memory_set.push(MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
            ), None);

        println!("mapping .data section");
        memory_set.push(MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
            ), None);

        println!("mapping .bss section");
        memory_set.push(MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
                ), None);
        
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
            (ekernel as usize).into(),
            MEMORY_END.into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);

        println!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                    MapArea::new(
                    (*pair).0.into(), 
                    ((*pair).0 + (*pair).1).into(), 
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                    ),
            None);
        }

        memory_set
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        // 获取memory_set
        let mut memory_set = Self::new_bare();

        // 设置跳板
        memory_set.map_trampoline();
        
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        // 判断魔术参数
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { 
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W; 
                }
                if ph_flags.is_execute() { 
                    map_perm |= MapPermission::X; 
                }

                let map_area = MapArea::new(
                    start_va, 
                    end_va,
                    MapType::Framed,
                    map_perm,
                );
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(map_area, 
                Some(&elf.input[ph.offset() as usize.. (ph.offset() + ph.file_size())  as usize]));
            }
        }

        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // 设置一个空页，防止死递归，一旦递归到空洞那么就会报页面异常
        user_stack_bottom += PAGE_SIZE;

        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;

        // 设置用户栈
        memory_set.push(MapArea::new(
            user_stack_bottom.into(), 
            user_stack_top.into(),
            MapType::Framed, 
            MapPermission::R | MapPermission::W | MapPermission::U),
             None);

        // 跳板下trap context块
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W
            )
            , None);

        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }

    // 激活sv39分页模式
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    // 通过vpn查询到pageTableEntry，也即是实际的物理页
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    #[allow(unused)]
    pub fn shrink_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self
            .areas
            .iter_mut()
            .find(|area| area.vpn_range.get_start() == start.floor())
        {
            area.shrink_to(&mut self.page_table, new_end.ceil());
            true
        } else{
            false
        }
    }

    #[allow(unused)]
    pub fn append_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self
            .areas
            .iter_mut()
            .find(|area| area.vpn_range.get_start() == start.floor())
        {
            area.append_to(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }

    
}



// 一段逻辑上连续的虚拟内存
pub struct MapArea {
    vpn_range: VPNRange,    // 连续虚拟内存页
    data_frames: BTreeMap<VirtPageNum, FrameTracker>, // 虚拟内存和物理内存的关联页map
    map_type: MapType,  // 访问方式
    map_perm: MapPermission, // 权限
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();

        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    // 在区域中加上一页虚拟内存
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;

        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0)
            }
            MapType::Framed => {
                let frame: FrameTracker = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }

        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    // 在虚拟内存中释放一页虚拟内存
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }

            _ => {}
        }

        page_table.unmap(vpn);
    }

    // 将当前区域的虚拟内存加入到新的pageTable
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    // 将虚拟内存从pageTable中释放
    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    #[allow(unused)]
    pub fn shrink_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for vpn in VPNRange::new(new_end, self.vpn_range.get_end()) {
            self.unmap_one(page_table, vpn)
        }
        self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
    }

    #[allow(unused)]
    pub fn append_to(&mut self, page_table: &mut PageTable, new_end:VirtPageNum) {
        for vpn in VPNRange::new(self.vpn_range.get_end(), new_end){
            self.map_one(page_table, vpn)
        }
        self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
    }

    // 将数据往连续的虚拟空间里面复制
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);

        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];

            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];

            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            
            if start >= len {
                break;
            }

            current_vpn.step();
        }
    }

}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical, // 虚拟页号对于物理页号
    Framed, // pageTable映射
}

// 地址空间：一系列有关联的逻辑段


bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) /2 ).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();

    assert!(
        !kernel_space.page_table.translate(mid_text.floor()).unwrap().writeable(),
    );

    println!("mid_rodata:{:?}", mid_rodata);
    assert!(
        !kernel_space.page_table.translate(mid_rodata.floor()).unwrap().writeable(),
    );

    assert!(
        !kernel_space.page_table.translate(mid_data.floor()).unwrap().executeable(),
    );

    println!("remap_test passed!");
}

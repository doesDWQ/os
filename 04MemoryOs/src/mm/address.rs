// 该文件已人工核对过

use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};



const PA_WIDTH_SV39: usize = 56;    // 总地址长度
const VA_WIDTH_SV39: usize = 39;    // 39位地址
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;   // 实际页号存储范围
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;   // 虚拟页号存储范围

// 物理地址
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct PhysAddr(pub usize);

//  虚拟地址
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

// 物理页号
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct PhysPageNum(pub usize);

// 虚拟页号
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct VirtPageNum(pub usize);

// debug 实现
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}


// 通过usize取出有用的56位物理地址
impl From<usize> for PhysAddr {
    fn from(v:usize) -> Self {
        Self(v & ( (1 << PA_WIDTH_SV39) -1))
    }
}

// 通过usize取出有用的虚拟地址号
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ( (1 << PPN_WIDTH_SV39) -1  ))
    }
}

// 通过usize取出有用的物理地址
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self (v & (1 << PA_WIDTH_SV39) - 1)
    }
}

// 通过usize取出有用的物理号
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v &((1 << VPN_WIDTH_SV39 ) -1))
    }
}

// 从物理地址中获取到usize值
impl From<PhysAddr> for usize {
    fn from(v:PhysAddr) -> Self {
        v.0
    }
}

// 从虚拟页号中获取usize值
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

// 物理地址获取对于的usize值
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!(1 << VA_WIDTH_SV39) -1)
        } else {
            v.0
        }
    }
}

// 需要页号获取对应的usize值
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}


impl VirtAddr {
    // 向下取出对应的虚拟地址号
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    // 向上取出对应的虚拟地址号
    pub fn ceil (&self) -> VirtPageNum {
        if self.0==0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }

    // 获取页标
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}


// 物理地址获取到物理地址号
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset() , 0);
        v.floor()
    }
}

// 虚拟号获取到虚拟地址
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}


impl PhysAddr {
    // 向下取整获取到物理地址号
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    // 向上取整获取到物理地址号
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE -1)  / PAGE_SIZE)
    }

    // 获取页标
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

// 物理地址转换为物理页号
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        // 物理地址不能为0
        assert_eq!(v.page_offset(), 0);
        // 向下取整获取物理页号
        v.floor()
    }
}

// 物理页号转换为物理地址
impl From<PhysPageNum> for PhysAddr { 
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    } 
}

impl VirtPageNum {
    // 通过虚拟页号，获取三级表
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];

        for i in(0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9
        }

        idx
    }
}


impl PhysPageNum {
    // 获取页表项
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)}
    }

    // 获取物理内存页指向的字节数组
    pub fn get_bytes_array(&self) -> &'static mut[u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)}
    }

    // 获取指定类型的可变数据的引用
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        // 裸指针解引用
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}


pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }

    pub fn get_start(&self) -> T {
        self.l
    }

    pub fn get_end (&self) -> T {
        self.r
    }
}

impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}

pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T, 
    end: T,
}

impl<T> SimpleRangeIterator<T> 
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r:T ) -> Self {
        Self{current:l, end:r}
    }
}

impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

// 虚拟页号迭代器
pub type VPNRange = SimpleRange<VirtPageNum>;
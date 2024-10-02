

use super::{get_block_cache, BlockDevice, BLOCK_SZ};
use alloc::sync::Arc;

type BitmapBlock = [u64; 64]; // 刚好对应一个缓存块的大小，512bytes 512字节，4096bits
const BLOCK_BITS: usize = BLOCK_SZ * 8; // 512 * 8 bit 也是512字节，4096bits

// 位图用来标记数据块分配情况
pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

// 分解成三级, (缓存块位置相对偏离位置, 缓冲块中u64的位置，u64下的具体位置)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit = bit % BLOCK_BITS;
    (block_pos, bit /64, bit %64)
}


impl Bitmap {
    // 新建一个位图
    pub fn new(start_block_id:usize, blocks:usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    // 申请一个空闲块
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id as usize, 
                Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                .iter()
                .enumerate()
                .find(|(_, bits64)| **bits64 != u64::MAX)
                .map(|(bits64_pos, bits64)| 
                    (bits64_pos, bits64.trailing_ones() as usize) 
                ) {
                    // 找到不为0的位置，设置为1
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    // 返回当前位置
                    Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
                } else {
                    None
                }
            });
            // 返回位置
            if pos.is_some() {
                return pos;
            }
        }

        None
    }

    // 释放一个空闲块
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);

        get_block_cache(block_pos + self.start_block_id,
            Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                // assert! 宏用于在运行时验证某个条件是否为真。如果条件为假，它会导致程序立即停止并打印错误信息
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                // 将对应位置设置为0
                bitmap_block[bits64_pos] -= 1u64 << inner_pos;
            })
    }

    // 获取当前最大容量
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
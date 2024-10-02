

use core::any::Any;


// 块数据抽象接口
pub trait BlockDevice: Send + Sync + Any {
    // 在磁盘中读取数据
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    // 往磁盘里面写数据
    fn write_block(&self, block_id: usize, buf: &[u8]);
}




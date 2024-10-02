use super::{BlockDevice, BLOCK_SZ};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

// 块缓存
pub struct BlockCache {
    cache: [u8; BLOCK_SZ], // 512 bytes  512字节
    block_id: usize,    // 块id
    block_device: Arc<dyn BlockDevice>,   // 块驱动
    modified: bool, // 标记当前块是否发生了改动
}

// 实现块缓存，最底层操作
impl BlockCache {
    // 获取一块块缓存
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
    	// 初始化cache
        let mut cache = [0u8; BLOCK_SZ];
        // 读取到所需要的数据
        block_device.read_block(block_id, &mut cache);

        // 返回blockCache对象
        Self {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }

    // 通过下标获取偏移指针地址
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    // 获取缓存数据中的不可变引用
    pub fn get_ref<T>(&self, offset: usize) -> &T 
    where 
    	T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
	
        unsafe { &*(addr as *const T) }
    }

    // 获取缓存可变引用
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T 
    where 
    	T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);

        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    // 对象释放时调用
    pub fn sync(&mut self) {
        if self.modified{
            // 将改动设置为false
            self.modified = false;
            // 将改动写入对应的缓存中
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }

    // 读取数据
    pub fn read<T, V>(&self, offset:usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    // 修改数据
    pub fn modify<T,V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
}

// 释放块缓存切片
impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}

const BLOCK_CACHE_SIZE: usize = 16; // 限制缓存块的数量


// 块缓存管理器
pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    // new一个块缓存管理器
    pub fn new() -> Self {
        Self { 
		    queue: VecDeque::new(),
	    }
    }

    // 根据block_id获取缓存，如果不存在读入缓存中
    pub fn get_block_cache(
    	&mut self, 
        block_id:usize, 
        block_device:Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id ) {
            Arc::clone(&pair.1)
        } else {
            // 如果缓存已经满了
            if self.queue.len() == BLOCK_CACHE_SIZE {
                if let Some((idx, _)) = self
                .queue
                .iter()
                .enumerate()
                .find(|(_, pair)| Arc::strong_count(&pair.1) == 1) 
		{
                    // 当引用只有1的时候释放掉该节点
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }

            let block_cache = Arc::new(Mutex::new(
	    	BlockCache::new(	
		    block_id,
		Arc::clone(&block_device)),
            ));
    
            // 将blockcache写入缓存队列
            self.queue.push_back((block_id, Arc::clone(&block_cache)));
    
            block_cache
        }
       
    }
}

lazy_static!{
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> = 
    	Mutex::new(BlockCacheManager::new());
}

// 获取块缓存
pub fn get_block_cache(
    block_id: usize,
    block_device: Arc<dyn BlockDevice>
) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MANAGER
    .lock()
    .get_block_cache(block_id, block_device)
}

// 刷新所有块缓存
pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}
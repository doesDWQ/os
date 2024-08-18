
mod virtio_blk;
use alloc::sync::Arc;
use easy_fs::BlockDevice;
use lazy_static::*;
use crate::board::BlockDeviceImpl;
pub use virtio_blk::VirtIOBlock;

#[cfg(feature = "board_qemu")]
type BlockDeviceImpl = virtio_blk::VirtIOBlock;

#[cfg(feature = "board_k210")]
type BlockDeviceImpl = sdcard::SDCardWrapper;


lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}


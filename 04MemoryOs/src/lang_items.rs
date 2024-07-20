// 该文件已人工核对过

use crate::sbi::shutdown;
use core::panic::PanicInfo;
use log::*;


#[panic_handler]
fn panic (info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!("Panicked at file: {}, line: {}, message: {}", location.file(), location.line(), info.message().unwrap());
    } else {
        error!("Panicked: {}", info.message().unwrap());
    }

    shutdown(true)
}
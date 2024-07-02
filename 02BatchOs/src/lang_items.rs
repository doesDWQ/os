
use core::panic::PanicInfo;
use log::*;
use super::*;
use sbi::shutdown;

#[panic_handler]
fn panic (info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("Panicked at file: {}, line: {}, message: {}", location.file(), location.line(), info.message().unwrap());
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }

    shutdown(true)
}
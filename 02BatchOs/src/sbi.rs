

use sbi_rt::Shutdown;


// 打印字符函数
pub fn console_putchar(c: usize) {
    sbi_rt::legacy::console_putchar(c);
}

// 关机函数
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }

    unreachable!()
}


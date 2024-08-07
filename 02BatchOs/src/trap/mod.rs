use core::arch::global_asm;


use riscv::register::{
    scause::{self, Exception, Trap}, 
    stval, 
    stvec, 
    mtvec::TrapMode
};
use crate::{batch::run_next_app, syscall::syscall};

mod context;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps(); // 加载陷入定义函数
    }

    unsafe {
        // trap时，直接跳入陷入定义函数处理
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

// 陷入定义函数
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // 读取陷入原因
    let stval: usize = stval::read();    // 读取陷入附加信息

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;   // 向下加4个字节，跳过异常指令
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception( Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }

        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }

    cx
}


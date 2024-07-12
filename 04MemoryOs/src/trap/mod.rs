use core::arch::{asm, global_asm};


use riscv::register::{
    mtvec::TrapMode, scause::{self, Exception, Interrupt, Trap}, sie, stval, stvec
};
use crate::{config::{TRAMPOLINE, TRAP_CONTEXT}, syscall::syscall, task::{current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next}, timer::set_next_trigger};

mod context;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

// pub fn init() {
//     extern "C" {
//         fn __alltraps(); // 加载陷入定义函数
//     }

//     unsafe {
//         // trap时，直接跳入陷入定义函数处理
//         stvec::write(__alltraps as usize, TrapMode::Direct);
//     }
// }

// 陷入定义函数
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    set_kernel_trap_entry();

    let cx: &mut TrapContext = current_trap_cx();
    let scause = scause::read(); // 读取陷入原因
    let stval: usize = stval::read();    // 读取陷入附加信息

    // println!("q23");

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;   
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception( Exception::StoreFault) 
            | Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::LoadFault)
            | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }

        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }

    trap_return();
}


#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();

    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    extern "C" {
        fn __alltraps();
        fn __restore();
    }

    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;

    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        )
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize,  TrapMode::Direct)
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize , TrapMode::Direct)
    }
}

pub fn init() {
    set_kernel_trap_entry();
}

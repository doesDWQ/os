

use core::arch::asm;
use crate::trap::TrapContext;



const User_stack_size: usize = 4096 * 2;
const Kerner_stack_size: usize = 4096 * 2;
const Max_app_num: usize = 16;  
const App_base_address: usize = 0x80400000;
const App_size_limit: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; Kerner_stack_size],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; User_stack_size],
}

static Kernel_stack: KernelStack = KernelStack {
    data: [0; Kerner_stack_size],
};

static User_stack: UserStack = UserStack {
    data: [0; User_stack_size],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + Kerner_stack_size
    }

    // 将上下文设置到内核栈中
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        unsafe {
            *cx_ptr = cx;
        }

        unsafe {
            cx_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    // 获取当前用户栈sp
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + User_stack_size
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; Max_app_num + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);

        for i in 0..self.num_app {
            println!("[kernel] app_{} [{:#x}, {:#x}]", i, self.app_start[i], self.app_start[i+1]);
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            use crate::sbi::shutdown;
            shutdown(false);
        }

        println!("[kernel] Loading app_{}", app_id);

        core::slice::from_raw_parts_mut(App_base_address as *mut u8, App_size_limit).fill(0);

        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );

        let app_dst = core::slice::from_raw_parts_mut(App_base_address as *mut u8, app_src.len());

        app_dst.copy_from_slice(app_src);

        asm!("fence.i");
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }

}

use lazy_static::*;
use riscv::register::mcause::Trap;
use crate::sync::UPSafeCell;

lazy_static! {
    static ref App_manager: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }

            let num_app_ptr = _num_app as usize as *const usize;

            let num_app = num_app_ptr.read_volatile();

            let mut app_start: [usize; Max_app_num + 1] = [0; Max_app_num + 1];

            let app_start_raw: &[usize] = 
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);

            app_start[..=num_app].copy_from_slice(app_start_raw);

            AppManager {
                num_app,
                current_app:0,
                app_start,
            }
        })
    };
}

pub fn init() {
    print_app_info();
}

pub fn print_app_info() {
    App_manager.exclusive_access().print_app_info();
}

pub fn run_next_app() -> ! {
    let mut app_manager = App_manager.exclusive_access();
    let current_app: usize = app_manager.get_current_app();

    unsafe {
        app_manager.load_app(current_app);
    }

    app_manager.move_to_next_app();
    drop(app_manager);

    extern "C" {
        fn __restore(cx_addr: usize);
    }

    unsafe {
        // 恢复调用栈
        let cx = Kernel_stack.push_context(
            TrapContext::app_init_context(App_base_address,User_stack.get_sp()
            )
        ) as *const _ as usize;
        __restore(cx);
    }
    
    println!("hello kitty");
    panic!("Unreachable in batch::run_crrent_app!");
}


use core::arch::asm;
use crate::trap::TrapContext;



pub const User_stack_size: usize = 4096 * 2;
pub const Kerner_stack_size: usize = 4096 * 2;
pub const Max_app_num: usize = 16;  
pub const App_base_address: usize = 0x80400000;
pub const App_size_limit: usize = 0x20000;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; Kerner_stack_size],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; User_stack_size],
}

static Kernel_Stack: [KernelStack; Max_app_num] = [
    KernelStack {
        data: [0; Kerner_stack_size],
    };Max_app_num
];

static User_Stack: [UserStack; Max_app_num] = [
    UserStack {
        data: [0; User_stack_size],
    };Max_app_num
];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + Kerner_stack_size
    }

    // 将上下文设置到内核栈中
    pub fn push_context(&self, cx: TrapContext) -> usize {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        unsafe {
            *cx_ptr = cx;
        }

       cx_ptr as usize
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
    // app_start: [usize; Max_app_num + 1],
}

impl AppManager {
    // pub fn print_app_info(&self) {
    //     println!("[kernel] num_app = {}", self.num_app);

    //     for i in 0..self.num_app {
    //         println!("[kernel] app_{} [{:#x}, {:#x}]", i, self.app_start[i], self.app_start[i+1]);
    //     }
    // }

    pub fn get_base_i(&self, app_id: usize) -> usize {
        App_base_address + app_id * App_size_limit
    }

    pub fn get_num_app(&self) -> usize {
        extern "C" {
            fn _num_app();
        }

        unsafe { (_num_app as usize as *const usize).read_volatile() }
    }

    pub fn load_apps(&self) {
        extern "C" { fn _num_app(); }
    
        let num_app_ptr = _num_app as usize as *const usize;
    
        let num_app = self.get_num_app();
    
        let app_start = unsafe {
            core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
        };
    
        for i in 0..num_app {
            let base_i = self.get_base_i(i);

            println!("load:i:{},base_i:{:x}", i, base_i);
    
            (base_i..base_i + App_size_limit).for_each(|addr| unsafe {
                (addr as *mut u8).write_volatile(0)
            });
    
            let src = unsafe {
                core::slice::from_raw_parts(
                    app_start[i] as *const u8, 
                    app_start[i+1] - app_start[i]
                )
            };
    
            let dst = unsafe {
                core::slice::from_raw_parts_mut(base_i as *mut u8, src.len())
            };
            dst.copy_from_slice(src);
    
            // 刷新缓存
            unsafe {
                asm!("fence.i");
            }
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }

}

use lazy_static::*;
use crate::sync::UPSafeCell;

lazy_static! {
    static ref App_manager: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }

            let num_app_ptr = _num_app as usize as *const usize;

            let num_app = num_app_ptr.read_volatile();

            // let mut app_start: [usize; Max_app_num + 1] = [0; Max_app_num + 1];

            // let app_start_raw: &[usize] = 
                // core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);

            // app_start[..=num_app].copy_from_slice(app_start_raw);

            AppManager {
                num_app,
                current_app:0,
                // app_start,
            }
        })
    };
}

pub fn init() {
    // print_app_info();
    let app_manager = App_manager.exclusive_access();
    app_manager.load_apps();
    drop(app_manager);
}

// pub fn print_app_info() {
//     // App_manager.exclusive_access().print_app_info();
// }

// pub fn run_next_app() -> ! {
//     let mut app_manager = App_manager.exclusive_access();
//     let current_app: usize = app_manager.get_current_app();
//     let entry = app_manager.get_base_i(current_app);

    

    
//     app_manager.move_to_next_app();

//     if current_app == app_manager.get_num_app() {
//         loop {}
//     }

//     println!("entry{:x}", entry);

//     drop(app_manager);

//     extern "C" {
//         fn __restore(cx_addr: usize);
//     }


//     unsafe {
//         // 压入上下文
//         let cx = Kernel_stack.push_context(
//             TrapContext::app_init_context(entry, User_stack.get_sp())
//         ) as *const _ as usize;
//         __restore(cx);
//     }
    
//     println!("hello kitty");
//     panic!("Unreachable in batch::run_crrent_app!");
// }


pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }

    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn init_app_cx(app_id: usize) -> usize {
    Kernel_Stack[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        User_Stack[app_id].get_sp(),
    ))
}

pub fn get_base_i(app_id: usize) -> usize {
    App_base_address + app_id * App_size_limit
}
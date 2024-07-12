use riscv::register::sstatus::{self, Sstatus, SPP};


#[repr(C)]
pub struct TrapContext {
    // 保存32个 x 寄存器
    pub x: [usize; 32],

    // csr sstaus寄存器
    pub sstatus: Sstatus,

    // csr sepc
    pub sepc: usize,

    pub kernel_satp: usize,

    pub kernel_sp: usize,

    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_sp (&mut self, sp:usize) {
        self.x[2] = sp;
    }

    // 初始化当前context 上下文
    pub fn app_init_context(
        entry: usize, 
        sp:usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        // 获取当前特权等级
        let mut sstatus = sstatus::read();
        // 修改为用户特权级别
        sstatus.set_spp(SPP::User);

        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc:entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };

        // 设置sp指针
        cx.set_sp(sp);
        cx
    }
}
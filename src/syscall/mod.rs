use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::gdt::Selectors;
use crate::task::context::Context;

pub fn init() {
    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::from_ptr(syscall_handler as *const ()));

    let (code_selector, data_selector) = Selectors::get_kernel_segments();
    let (user_code_selector, user_data_selector) = Selectors::get_user_segments();

    Star::write(
        user_code_selector,
        user_data_selector,
        code_selector,
        data_selector,
    )
    .unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[naked]
extern "C" fn syscall_handler() {
    unsafe {
        core::arch::naked_asm!(
            "sub rsp, 0x38",
            crate::push_context!(),

            // Move the 4th argument in r10 to rcx to fit the C ABI
            "mov rdi, rsp",
            "call {syscall_matcher}",

            crate::pop_context!(),
            "add rsp, 0x38",

            "sysretq",
            syscall_matcher = sym syscall_matcher,
        );
    }
}

use self::op::*;
use sc::nr::*;

const SYS_PUT_STRING: usize = 10000;
const SYS_MALLOC: usize = 10001;

fn syscall_matcher(regs: &mut Context) {
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r10;
    let arg5 = regs.r8;
    let arg6 = regs.r9;

    let syscall_num = regs.rax;

    let ret = match syscall_num {
        SCHED_YIELD => sys_yield(),
        EXIT => sys_exit(arg1),
        WAIT4 => sys_wait4(arg1),
        FORK => sys_fork(),
        VFORK => sys_fork(),

        OPEN => sys_open(arg1, arg2, arg3),
        CLOSE => sys_close(arg1),
        READ => sys_read(arg1, arg2, arg3),
        WRITE => sys_write(arg1, arg2, arg3),
        LSEEK => sys_lseek(arg1, arg2),
        FSTAT => sys_fstat(arg1, arg2),

        SYS_PUT_STRING => sys_putstring(arg1, arg2),
        SYS_MALLOC => sys_malloc(arg1, arg2),
        _ => -1,
    };

    regs.rax = ret as usize;
}

pub mod op;

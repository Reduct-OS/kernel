use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::gdt::Selectors;

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
            "push rcx",
            "push r11",

            // Move the 4th argument in r10 to rcx to fit the C ABI
            "mov rcx, r10",
            "call {syscall_matcher}",

            "jmp ret_from_syscall",
            syscall_matcher = sym syscall_matcher,
        );
    }
}

use self::op::*;
use sc::nr::*;

const SYS_PUT_STRING: usize = 10000;
const SYS_MALLOC: usize = 10001;

fn syscall_matcher(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let syscall_num: usize;
    unsafe { core::arch::asm!("mov {0}, rax", out(reg) syscall_num) };

    let ret = match syscall_num {
        SCHED_YIELD => sys_yield(),
        EXIT => sys_exit(arg1),
        WAIT4 => sys_wait4(arg1),

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

    ret
}

pub mod op;

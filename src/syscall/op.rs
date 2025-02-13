use alloc::sync::Arc;

use crate::{irq::InterruptIndex, serial_print, task::scheduler::SCHEDULER};

pub fn sys_yield() -> isize {
    unsafe {
        core::arch::asm!(
            "int {interrupt_number}",
            interrupt_number =
            const InterruptIndex::Timer as u8
        );
    }

    0
}

pub fn sys_exit(code: usize) -> isize {
    let process = SCHEDULER
        .lock()
        .current()
        .upgrade()
        .and_then(|thread| thread.read().process.upgrade());

    if let Some(process) = process {
        let mut scheduler = SCHEDULER.lock();
        for thread in process.read().threads.iter() {
            scheduler.remove(Arc::downgrade(thread));
        }
        process.read().exit();
    }

    sys_yield()
}

pub fn sys_putstring(addr: usize, len: usize) -> isize {
    if let Ok(str) = unsafe { str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len)) }
    {
        serial_print!("{}", str);
        return str.len() as isize;
    }

    0
}

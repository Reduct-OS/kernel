use alloc::{string::ToString, sync::Arc};
use x86_64::VirtAddr;

use crate::{
    fs::operation::OpenMode,
    irq::InterruptIndex,
    memory::{MappingType, MemoryManager, ref_current_page_table},
    serial_print,
    task::{
        context::Context,
        get_current_thread,
        process::{PROCESSES, ProcessId},
        scheduler::SCHEDULER,
    },
};

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

pub fn sys_wait4(pid: usize) -> isize {
    loop {
        if PROCESSES
            .read()
            .iter()
            .find(|process| process.read().id == ProcessId::from(pid as u64))
            .is_none()
        {
            break;
        }

        sys_yield();
    }

    pid as isize
}

pub fn sys_malloc(addr: usize, len: usize) -> isize {
    if MemoryManager::alloc_range(
        VirtAddr::new(addr as u64),
        len as u64,
        MappingType::UserData.flags(),
        &mut ref_current_page_table(),
    )
    .is_err()
    {
        return -1;
    }

    0
}

pub fn sys_pipe(fd: usize) -> isize {
    let fd = unsafe { core::slice::from_raw_parts_mut(fd as *mut usize, 2) };

    if let Some(ret) = crate::fs::operation::pipe(fd) {
        return ret as isize;
    }

    -1
}

pub fn sys_open(path: usize, mode: usize, len: usize) -> isize {
    let openmode = OpenMode::from(mode);

    let path = str::from_utf8(unsafe { core::slice::from_raw_parts(path as *const u8, len) }).ok();

    if let Some(path) = path {
        if let Some(ret) = crate::fs::operation::open(path.to_string(), OpenMode::from(mode)) {
            return ret as isize;
        }
    }

    -1
}

pub fn sys_close(fd: usize) -> isize {
    if let Some(ret) = crate::fs::operation::close(fd) {
        return 0;
    }

    -1
}

pub fn sys_read(fd: usize, buf: usize, len: usize) -> isize {
    crate::fs::operation::read(fd, unsafe {
        core::slice::from_raw_parts_mut(buf as *mut u8, len)
    }) as isize
}

pub fn sys_write(fd: usize, buf: usize, len: usize) -> isize {
    crate::fs::operation::write(fd, unsafe {
        core::slice::from_raw_parts(buf as *const u8, len)
    }) as isize
}

pub fn sys_lseek(fd: usize, offset: usize) -> isize {
    if crate::fs::operation::lseek(fd, offset).is_none() {
        return -1;
    }
    offset as isize
}

pub fn sys_fstat(fd: usize, buf: usize) -> isize {
    if crate::fs::operation::fstat(fd, buf).is_none() {
        return -1;
    }
    0
}

pub fn sys_fork(regs: &mut Context) -> isize {
    let current_thread = get_current_thread();
    current_thread.read().fork_thread(regs)
}

use alloc::sync::{Arc, Weak};
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;

use super::context::Context;
use super::get_current_process;
use super::process::{KERNEL_PROCESS, WeakSharedProcess};
use super::scheduler::SCHEDULER;
use super::stack::{KernelStack, UserStack};
use crate::gdt::Selectors;
use crate::memory::{ExtendedPageTable, KERNEL_PAGE_TABLE};

core::arch::global_asm!(include_str!("../syscall/ret_syscall.S"));

unsafe extern "C" {
    fn ret_from_syscall();
}

pub(super) type SharedThread = Arc<RwLock<Thread>>;
pub(super) type WeakSharedThread = Weak<RwLock<Thread>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Thread {
    pub id: ThreadId,
    pub kernel_stack: KernelStack,
    pub context: Context,
    pub process: WeakSharedProcess,
    pub sleeping: bool,
}

impl Thread {
    pub fn new(process: WeakSharedProcess) -> Self {
        Self {
            id: ThreadId::new(),
            context: Context::default(),
            kernel_stack: KernelStack::default(),
            process,
            sleeping: false,
        }
    }

    pub fn get_init_thread() -> WeakSharedThread {
        let thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));
        let thread = Arc::new(RwLock::new(thread));
        KERNEL_PROCESS.write().threads.push(thread.clone());
        Arc::downgrade(&thread)
    }

    pub fn new_kernel_thread(function: fn()) {
        let mut thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));

        thread.context.init(
            function as usize,
            thread.kernel_stack.end_address(),
            KERNEL_PAGE_TABLE.lock().physical_address(),
            Selectors::get_kernel_segments(),
        );

        let thread = Arc::new(RwLock::new(thread));
        KERNEL_PROCESS.write().threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread));
    }

    pub fn new_user_thread(process: WeakSharedProcess, entry_point: usize) {
        let mut thread = Self::new(process.clone());
        let process = process.upgrade().unwrap();
        let mut process = process.write();
        UserStack::map(&mut process.page_table);

        thread.context.init(
            entry_point,
            UserStack::end_address(),
            process.page_table.physical_address(),
            Selectors::get_user_segments(),
        );

        let thread = Arc::new(RwLock::new(thread));
        process.threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread));
    }

    pub fn fork_thread(&self) {
        let current_process = get_current_process();

        let mut thread = Self::new(Arc::downgrade(&current_process));
        let mut process = current_process.write();

        thread.context.copy_from(self.context);
        thread.context.init(
            ret_from_syscall as usize,
            UserStack::end_address(),
            process.page_table.physical_address(),
            Selectors::get_user_segments(),
        );
        thread.context.rcx = self.context.rip;

        let thread = Arc::new(RwLock::new(thread));
        process.threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread));
    }
}

use core::ops::Deref;

use alloc::{string::String, sync::Arc};
use spin::RwLock;
use x86_64::VirtAddr;

use crate::{
    memory::ExtendedPageTable,
    task::{process::ProcessId, scheduler::SCHEDULER},
};

use super::{
    USER_FS_MANAGER,
    vfs::inode::{Inode, InodeRef},
};

pub struct UserCommand {
    pub cmd: usize,
    pub offset: usize,
    pub buf_addr: usize,
    pub buf_size: usize,
    ok_signal: usize,
}

impl UserCommand {
    pub fn new(cmd: usize, offset: usize, buf_addr: usize, buf_size: usize) -> UserCommand {
        Self {
            cmd,
            offset,
            buf_addr,
            buf_size,
            ok_signal: 0,
        }
    }
}

impl Deref for UserCommand {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self as *const UserCommand as *const u8, size_of::<Self>())
        }
    }
}

pub struct UserFS {
    path: String,
    pid: ProcessId,
}

impl UserFS {
    pub fn new(pid: ProcessId) -> InodeRef {
        Arc::new(RwLock::new(Self {
            path: String::new(),
            pid,
        }))
    }
}

impl Inode for UserFS {
    fn when_mounted(&mut self, path: String, father: Option<super::vfs::inode::InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![0u8; buf.len()];
            let command = UserCommand::new(1, offset, buffer.as_mut_ptr() as usize, buffer.len());

            let process = SCHEDULER.lock().find(self.pid);
            if let Some(process) = process {
                let process = process.upgrade().unwrap();
                let proc_page_table = &process.read().page_table;
                proc_page_table.write_to_mapped_address(&command, VirtAddr::new(fs_addr as u64));

                let ok_signal: &mut [usize; 1] = &mut [0; 1];

                while ok_signal[0] == 0 {
                    proc_page_table.read_mapped_address(
                        unsafe {
                            core::slice::from_raw_parts_mut(
                                ok_signal.as_mut_ptr() as *mut u8,
                                ok_signal.len() * size_of::<usize>(),
                            )
                        },
                        VirtAddr::new(
                            fs_addr as u64 + core::mem::offset_of!(UserCommand, ok_signal) as u64,
                        ),
                    );

                    crate::syscall::op::sys_yield();
                }

                buf.copy_from_slice(&buffer);
            }

            drop(buffer);
        }

        usize::MAX
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![0u8; buf.len()];
            buffer.copy_from_slice(buf);
            let command = UserCommand::new(2, offset, buffer.as_mut_ptr() as usize, buffer.len());

            let process = SCHEDULER.lock().find(self.pid);
            if let Some(process) = process {
                let process = process.upgrade().unwrap();
                let proc_page_table = &process.read().page_table;
                proc_page_table.write_to_mapped_address(&command, VirtAddr::new(fs_addr as u64));

                let ok_signal: &mut [usize; 1] = &mut [0; 1];

                while ok_signal[0] == 0 {
                    proc_page_table.read_mapped_address(
                        unsafe {
                            core::slice::from_raw_parts_mut(
                                ok_signal.as_mut_ptr() as *mut u8,
                                ok_signal.len() * size_of::<usize>(),
                            )
                        },
                        VirtAddr::new(
                            fs_addr as u64 + core::mem::offset_of!(UserCommand, ok_signal) as u64,
                        ),
                    );

                    crate::syscall::op::sys_yield();
                }
            }

            drop(buffer);
        }

        usize::MAX
    }

    fn inode_type(&self) -> super::vfs::inode::InodeTy {
        super::vfs::inode::InodeTy::File
    }
}

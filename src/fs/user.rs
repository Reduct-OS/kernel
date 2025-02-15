use core::ops::{Deref, DerefMut};

use alloc::{string::String, sync::Arc, vec::Vec};
use spin::RwLock;
use x86_64::VirtAddr;

use crate::{
    memory::ExtendedPageTable,
    task::{process::ProcessId, scheduler::SCHEDULER},
};

use super::{
    USER_FS_MANAGER,
    operation::get_path_by_fd,
    vfs::inode::{FileInfo, Inode, InodeRef, InodeTy},
};

pub struct UserCommand {
    pub cmd: usize,
    pub offset: usize,
    pub buf_addr: usize,
    pub buf_size: usize,
    ok_signal: usize,
    pub ret_val: isize,
    pub ret_val2: isize,
    pub ret_val3: isize,
}

impl UserCommand {
    pub fn new(cmd: usize, offset: usize, buf_addr: usize, buf_size: usize) -> UserCommand {
        Self {
            cmd,
            offset,
            buf_addr,
            buf_size,
            ok_signal: 0,
            ret_val: -1,
            ret_val2: -1,
            ret_val3: -1,
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

impl DerefMut for UserCommand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut UserCommand as *mut u8, size_of::<Self>())
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

const USER_READ: usize = 1;
const USER_WRITE: usize = 2;
const USER_OPEN: usize = 3;
const USER_SIZE: usize = 4;
const USER_LIST: usize = 5;
const USER_IOCTL: usize = 6;

#[derive(Debug, Clone, Copy, Default)]
pub struct RetVecStruct {
    pub addr: usize,
    pub len: usize,
    pub cap: usize,
}

impl Deref for RetVecStruct {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self as *const RetVecStruct as *const u8, size_of::<Self>())
        }
    }
}

impl DerefMut for RetVecStruct {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut RetVecStruct as *mut u8, size_of::<Self>())
        }
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

    fn open(&self, name: String) -> Option<InodeRef> {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![0u8; name.as_bytes().len()];
            buffer.copy_from_slice(name.as_bytes());
            let command =
                UserCommand::new(USER_OPEN, 0, buffer.as_mut_ptr() as usize, buffer.len());

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

        None
    }

    fn read_at(&self, fd: usize, offset: usize, buf: &mut [u8]) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![0u8; buf.len()];
            let mut command = UserCommand::new(
                USER_READ,
                offset,
                buffer.as_mut_ptr() as usize,
                buffer.len(),
            );

            let path = get_path_by_fd(fd).unwrap();
            let path = path.as_str();
            command.ret_val = path.as_ptr() as isize;
            command.ret_val2 = path.len() as isize;

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

                proc_page_table.read_mapped_address(&mut command, VirtAddr::new(fs_addr as u64));
            }

            drop(buffer);

            return command.ret_val as usize;
        }

        usize::MAX
    }

    fn write_at(&self, fd: usize, offset: usize, buf: &[u8]) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![0u8; buf.len()];
            buffer.copy_from_slice(buf);
            let mut command = UserCommand::new(
                USER_WRITE,
                offset,
                buffer.as_mut_ptr() as usize,
                buffer.len(),
            );

            let path = get_path_by_fd(fd).unwrap();
            let path = path.as_str();
            command.ret_val = path.as_ptr() as isize;
            command.ret_val2 = path.len() as isize;

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

                proc_page_table.read_mapped_address(&mut command, VirtAddr::new(fs_addr as u64));
            }

            drop(buffer);

            return command.ret_val as usize;
        }

        usize::MAX
    }

    fn size(&self, fd: usize) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut command = UserCommand::new(USER_SIZE, 0, 0, 0);

            let path = get_path_by_fd(fd).unwrap();
            let path = path.as_str();
            command.ret_val = path.as_ptr() as isize;
            command.ret_val2 = path.len() as isize;

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

                proc_page_table.read_mapped_address(&mut command, VirtAddr::new(fs_addr as u64));
            }

            return command.ret_val as usize;
        }

        usize::MAX
    }

    fn list(&self, fd: usize) -> Vec<FileInfo> {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut command = UserCommand::new(USER_LIST, 0, 0, 0);

            let path = get_path_by_fd(fd).unwrap();
            let path = path.as_str();
            command.ret_val = path.as_ptr() as isize;
            command.ret_val2 = path.len() as isize;

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

                proc_page_table.read_mapped_address(&mut command, VirtAddr::new(fs_addr as u64));

                let ret_struct_addr = command.ret_val as usize;
                let ret_struct_len = command.ret_val2 as usize;
                let ret_struct_cap = command.ret_val3 as usize;
                let vec = unsafe {
                    Vec::from_raw_parts(
                        ret_struct_addr as *mut String,
                        ret_struct_len,
                        ret_struct_cap,
                    )
                };

                let mut result = Vec::new();
                for name in vec {
                    let file_info = FileInfo {
                        name,
                        ty: InodeTy::File,
                    };
                    result.push(file_info);
                }

                return result;
            }
        }

        Vec::new()
    }

    fn inode_type(&self) -> InodeTy {
        InodeTy::Dir
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> usize {
        let user_fs_manager = USER_FS_MANAGER.lock();
        let fs_addr = user_fs_manager.get(&self.pid);
        if let Some(&fs_addr) = fs_addr {
            let mut buffer = alloc::vec![cmd, arg];
            let mut command =
                UserCommand::new(USER_IOCTL, 0, buffer.as_mut_ptr() as usize, buffer.len());

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

                proc_page_table.read_mapped_address(&mut command, VirtAddr::new(fs_addr as u64));
            }

            drop(buffer);

            return command.ret_val as usize;
        }

        usize::MAX
    }
}

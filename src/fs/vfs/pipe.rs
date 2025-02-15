use alloc::{string::String, sync::Arc, vec::Vec};
use spin::{Mutex, RwLock};

use super::inode::{Inode, InodeRef};

pub struct PipeFS {
    path: String,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl PipeFS {
    pub fn new() -> InodeRef {
        let inode = Arc::new(RwLock::new(Self {
            path: String::new(),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }));
        inode
    }
}

impl Inode for PipeFS {
    fn when_mounted(&mut self, path: String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn inode_type(&self) -> super::inode::InodeTy {
        super::inode::InodeTy::File
    }

    fn read_at(&self, fd: usize, _offset: usize, buf: &mut [u8]) -> usize {
        while self.buffer.lock().is_empty() {
            crate::syscall::op::sys_yield();
        }
        buf.copy_from_slice(self.buffer.lock().as_slice());
        self.buffer.lock().clear();
        buf.len()
    }

    fn write_at(&self, fd: usize, _offset: usize, buf: &[u8]) -> usize {
        for &byte in buf {
            self.buffer.lock().push(0);
        }

        self.buffer.lock().len()
    }
}

use crate::acpi::BUF;
use alloc::{string::String, sync::Arc};
use spin::RwLock;

use super::inode::{Inode, InodeRef};

pub struct AcpiFS {
    path: String,
}

impl AcpiFS {
    pub fn new() -> InodeRef {
        let inode = Arc::new(RwLock::new(Self {
            path: String::new(),
        }));
        inode
    }
}

impl Inode for AcpiFS {
    fn when_mounted(&mut self, path: String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn read_at(&self, fd: usize, _offset: usize, buf: &mut [u8]) -> usize {
        let data = BUF.get().unwrap();

        buf.copy_from_slice(&data);
        data.len()
    }

    fn size(&self, _fd: usize) -> usize {
        let data = BUF.get().unwrap();
        data.len()
    }

    fn inode_type(&self) -> super::inode::InodeTy {
        super::inode::InodeTy::File
    }
}

use alloc::{string::String, sync::Arc, vec::Vec};
use spin::RwLock;

use crate::ref_to_mut;

use super::inode::{Inode, InodeRef};

pub struct PipeFS {
    path: String,
    buffer: Vec<u8>,
}

impl PipeFS {
    pub fn new() -> InodeRef {
        let inode = Arc::new(RwLock::new(Self {
            path: String::new(),
            buffer: Vec::new(),
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

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let mut idx: usize = 0;

        for (i, &byte) in self.buffer.iter().enumerate() {
            if (i + offset) > self.buffer.len() {
                break;
            }
            idx = i;
            buf[idx + offset] = byte;
        }

        idx
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> usize {
        for &byte in buf {
            ref_to_mut(self).buffer.push(byte);
        }

        buf.len()
    }
}
